use display_delegate::display;
use lifetime_remover::remove_lifetimes_from_path;
use proc_macro2::{Span, TokenNode, TokenStream, TokenTree};
use quote::{ToTokens};
use std::fmt::{Error, Formatter};
use std::mem;
use syn::{self, ArgCaptured, Block, Expr, ExprVerbatim, FnArg, Ident, Pat, PatIdent, PathSegment, Stmt};
use syn::punctuated::Punctuated;
use syn::token::{Comma, Colon2, Semi};

const ARGS_REPLACEMENT_TUPLE_NAME: &str  = "__mocktopus_args_replacement_tuple__";
const MOCKTOPUS_EXTERN_CRATE_NAME: &str = "__mocktopus_extern_crate_inside_header__";

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
    pub fn build(&self, fn_ident: &Ident, fn_args: &Punctuated<FnArg, Comma>, fn_block_span: Span) -> Stmt {
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
    }};
}}"#,
            mocktopus_crate         = MOCKTOPUS_EXTERN_CRATE_NAME,
            full_fn_name            = display(|f| write_full_fn_name(f, self, fn_ident)),
            args_tuple              = display(|f| write_args_tuple(f, fn_args)),
            args_replacement_tuple  = ARGS_REPLACEMENT_TUPLE_NAME,
            args_replacement        = display(|f| write_args_replacement(f, fn_args)),
            args_forget             = display(|f| write_args_forget(f, fn_args)));
        let header_token_stream = syn::parse_str::<Block>(&header_str)
            .expect(error_msg!("generated header unparsable"))
            .into_tokens()
            .into_iter()
            .map(|tt| make_token_tree_span_call_site(tt, fn_block_span))
            .collect();

        Stmt::Semi(
            Expr::Verbatim(
                ExprVerbatim {
                    tts: header_token_stream,
                }
            ),
            Semi([fn_block_span])
        )
    }
}

fn make_token_tree_span_call_site(mut token_tree: TokenTree, fn_block_span: Span) -> TokenTree {
    token_tree.span = fn_block_span;
    if let TokenNode::Group(_, ref mut token_stream) = token_tree.kind {
        *token_stream = mem::replace(token_stream, TokenStream::empty())
            .into_iter()
            .map(|tt| make_token_tree_span_call_site(tt, fn_block_span))
            .collect();
    }
    token_tree
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
               MOCKTOPUS_EXTERN_CRATE_NAME, fn_arg_name)?;
    }
    write!(f, ") }}")
}

fn write_args_replacement<T>(f: &mut Formatter, fn_args: &Punctuated<FnArg, T>) -> Result<(), Error> {
    if fn_args.is_empty() {
        return writeln!(f, "()");
    }
    writeln!(f, "unsafe {{")?;
    for (fn_arg_index, fn_arg_name) in iter_fn_arg_names(fn_args).enumerate() {
        writeln!(f, "::std::mem::replace({}::mocking_utils::as_mut(&{}), {}.{});",
                 MOCKTOPUS_EXTERN_CRATE_NAME, fn_arg_name, ARGS_REPLACEMENT_TUPLE_NAME, fn_arg_index)?;
    }
    writeln!(f, "}}")
}

fn write_args_forget<T>(f: &mut Formatter, fn_args: &Punctuated<FnArg, T>) -> Result<(), Error> {
    for fn_arg_name in iter_fn_arg_names(fn_args) {
        writeln!(f, "::std::mem::forget({});", fn_arg_name)?;
    }
    Ok(())
}

pub fn iter_fn_arg_names<'a, T>(input_args: &'a Punctuated<FnArg, T>) -> impl Iterator<Item = &'a str> {
    input_args.iter()
        .map(|fn_arg| match *fn_arg {
            FnArg::SelfRef(_) | FnArg::SelfValue(_) => "self",
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
            ) => ident.as_ref(),
            _ => panic!("{}: '{}'", error_msg!("invalid fn arg type"), fn_arg.clone().into_tokens()),
        })
}
