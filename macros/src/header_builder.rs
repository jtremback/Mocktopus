use display_delegate::display;
use lifetime_remover::remove_lifetimes_from_path;
use proc_macro2::Span;
use quote::{Tokens, ToTokens};
use std::fmt::{Error, Formatter};
use std::iter;
use syn::{self, ArgCaptured, FnArg, Ident, Pat, PatIdent, PathSegment, Stmt};
use syn::punctuated::Punctuated;
use syn::token::{Comma, Colon2};

const ARGS_REPLACEMENT_TUPLE_NAME: &str  = "__mocktopus_args_replacement_tuple__";
const MOCKTOPUS_EXTERN_CRATE_NAME: &str = "__mocktopus_extern_crate_inside_header__";

macro_rules! quote_call_site {
    ($($tt:tt)*) => (quote_spanned!(Span::call_site()=> $($tt)*));
}

macro_rules! error_msg {
    ($msg:expr) => { concat!("Mocktopus internal error: ", $msg) }
}

pub enum FnHeaderBuilder<'a> {
    StaticFn,
    StructImpl,
    TraitDefault,
    TraitImpl(&'a Punctuated<PathSegment, Colon2>),
}

impl<'a> FnHeaderBuilder<'a> {
    pub fn build(&self, fn_ident: &Ident, fn_args: &Punctuated<FnArg, Comma>) -> Stmt {
        let header_str = format!(
r#"{{
    extern crate mocktopus as {mocktopus_crate};
    match ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe (
            || {mocktopus_crate}::mocking::Mockable::call_mock(&{full_fn_name}, {args_tuple}))) {{
        Ok({mocktopus_crate}::mocking::MockResult::Continue({args_replacement_tuple})) => {args_replacement},
        Ok({mocktopus_crate}::mocking::MockResult::Return(result)) => return result,
        Err(unwind) => {{
            {args_forget}
            ::std::panic::resume_unwind(unwind);
        }}
    }}
}}"#,
            mocktopus_crate         = MOCKTOPUS_EXTERN_CRATE_NAME,
            full_fn_name            = display(|f| write_full_fn_name(f, self, fn_ident)),
            args_tuple              = display(|f| write_args_tuple(f, fn_args)),
            args_replacement_tuple  = ARGS_REPLACEMENT_TUPLE_NAME,
            args_replacement        = display(|f| write_args_replacement(f, fn_args)),
            args_forget             = display(|f| write_args_forget(f, fn_args)));
        syn::parse_str(&header_str).expect(error_msg!("generated header unparsable"))
    }
}

fn write_full_fn_name(f: &mut Formatter, builder: &FnHeaderBuilder, fn_ident: &Ident) -> Result<(), Error> {
    match *builder {
        FnHeaderBuilder::StaticFn => (),
        FnHeaderBuilder::StructImpl |
        FnHeaderBuilder::TraitDefault => write!(f, "Self::")?,
        FnHeaderBuilder::TraitImpl(ref path) => write!(f, "<Self as {}>::", display(|f| write_trait_path(f, path)))?,
    }
    write!(f, "{}", fn_ident.as_ref())
}

fn write_trait_path<T: ToTokens + Clone>(f: &mut Formatter, trait_path: &Punctuated<PathSegment, T>) -> Result<(), Error> {
    let mut trait_path_without_lifetimes = trait_path.clone();
    remove_lifetimes_from_path(&mut trait_path_without_lifetimes);
    write!(f, "{}", trait_path_without_lifetimes.into_tokens())
}

fn write_args_tuple<T>(f: &mut Formatter, fn_args: &Punctuated<FnArg, T>) -> Result<(), Error> {
    if fn_args.is_empty() {
        return write!(f, "()");
    }
    write!(f, "unsafe {{ (")?;
    for fn_arg_name in iter_fn_arg_names(fn_args) {
        write!(f, "::std::mem::replace({}::mocking_utils::as_mut(&{}), ::std::mem::uninitialized()), ",
               MOCKTOPUS_EXTERN_CRATE_NAME, fn_arg_name.to_string())?;
    }
    write!(f, ") }}")
}

fn write_args_replacement<T>(f: &mut Formatter, fn_args: &Punctuated<FnArg, T>) -> Result<(), Error> {
    if fn_args.is_empty() {
        return writeln!(f, "{}", quote_call_site!(()));
    }
    let mocktopus_extern_crate_name = iter::repeat(mocktopus_extern_crate_name());
    let fn_arg_names = iter_fn_arg_names(fn_args);
    let args_replacement_tuple_name = iter::repeat(args_replacement_tuple_name());
    let fn_arg_indexes = 0..;
    let result = quote_call_site!(
        unsafe {
            #(
                ::std::mem::replace(
                    #mocktopus_extern_crate_name::mocking_utils::as_mut(#fn_arg_names),
                    #args_replacement_tuple_name.#fn_arg_indexes);
            )*
        }
    );
    println!("{}\n\n", result);
    writeln!(f, "{}", result)


//    writeln!(f, "unsafe {{")?;
//    for (fn_arg_index, fn_arg_name) in iter_fn_arg_names(fn_args).enumerate() {
//        writeln!(f, "::std::mem::replace({}::mocking_utils::as_mut(&{}), {}.{});",
//                 MOCKTOPUS_EXTERN_CRATE_NAME, fn_arg_name.to_string(), ARGS_REPLACEMENT_TUPLE_NAME, fn_arg_index)?;
//    }
//    writeln!(f, "}}")
}

fn write_args_forget<T>(f: &mut Formatter, fn_args: &Punctuated<FnArg, T>) -> Result<(), Error> {
    let fn_arg_names_iter = iter_fn_arg_names(fn_args);
    let result = quote_call_site!(
        #(
            ::std::mem::forget(#fn_arg_names_iter);
        )*
    );
    writeln!(f, "{}", result)
}

pub fn iter_fn_arg_names<'a, T>(input_args: &'a Punctuated<FnArg, T>) -> impl Iterator<Item = Tokens> + 'a {
    input_args.iter()
        .map(|fn_arg| match *fn_arg {
            FnArg::SelfRef(_) | FnArg::SelfValue(_) => quote_call_site!(self),
            FnArg::Captured(
                ArgCaptured {
                    pat: Pat::Ident(
                        PatIdent {
                            ref ident,
                            subpat: None,
                            ..
                        }
                    ),
                    ..
                }
            ) => quote_call_site!(#ident),
            _ => panic!("{}: '{}'", error_msg!("invalid fn arg type"), fn_arg.clone().into_tokens()),
        })
}

fn args_replacement_tuple_name() -> Tokens {
    quote_call_site!(__mocktopus_args_replacement_tuple__)
}

fn mocktopus_extern_crate_name() -> Tokens {
    quote_call_site!(__mocktopus_extern_crate_inside_header__)
}
