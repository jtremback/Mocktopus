use header_builder::FnHeaderBuilder;
use syn::{ArgCaptured, Attribute, Block, FnArg, FnDecl, Ident, Item, ItemFn, ItemImpl, ItemMod, ItemTrait,
          MethodSig, Pat, PatIdent, TraitItem, TraitItemMethod};
use syn::punctuated::Punctuated;
use syn::token::{Comma, Const};

pub fn inject_item(item: &mut Item) {
    match *item {
        Item::Fn(ref mut item_fn)       => inject_fn(item_fn),
        Item::Mod(ref mut item_mod)     => inject_mod(item_mod),
        Item::Trait(ref mut item_trait) => inject_trait(item_trait),
        Item::Impl(ref mut item_impl)   => inject_impl(item_impl),
        _                               => (),
    }
}

fn inject_fn(item_fn: &mut ItemFn) {
    inject_any_fn(FnHeaderBuilder::StaticFn, &item_fn.attrs, &item_fn.constness, &item_fn.ident, &mut item_fn.decl,
                  &mut *item_fn.block);
}

fn inject_mod(item_mod: &mut ItemMod) {
    if is_not_mockable(&item_mod.attrs) {
        return
    }
    item_mod.content.iter_mut()
        .flat_map(|c| &mut c.1)
        .for_each(inject_item)
}

fn inject_trait(item_trait: &mut ItemTrait) {
    if is_not_mockable(&item_trait.attrs) {
        return
    }
    for item in &mut item_trait.items {
        if let TraitItem::Method(TraitItemMethod {
            ref attrs,
            ref mut sig,
            default: Some(ref mut block),
            ..
        }) = *item {
            inject_any_method(FnHeaderBuilder::TraitDefault, attrs, sig, block);
        }
    }
//    for item in items.iter_mut().filter(|i| do_item_attrs_let_injector_in(&i.attrs)) {
//        if let TraitItemKind::Method(
//            MethodSig {
//                unsafety: _,
//                ref constness,
//                abi: _,
//                ref mut decl,
//                generics: _},
//            Some(ref mut block)) = item.node {
//            let builder = HeaderBuilder::default()
//                .set_is_method();
//            inject_fn(builder, &item.ident, &mut decl.inputs, constness, block);
//        }
//    }
}

fn inject_impl(item_impl: &mut ItemImpl) {
    if is_not_mockable(&item_impl.attrs) {
        return
    }
//    for item in items.iter_mut().filter(|i| do_item_attrs_let_injector_in(&i.attrs)) {
//        if let ImplItemKind::Method(
//            MethodSig {
//                unsafety: _,
//                ref constness,
//                abi: _,
//                ref mut decl,
//                generics: _},
//            ref mut block) = item.node {
//            let builder = HeaderBuilder::default()
//                .set_is_method()
//                // full path with trait name is needed in impl of concrete struct to avoid ambiguity
//                .set_trait_path(trait_path);
//            inject_fn(builder, &item.ident, &mut decl.inputs, constness, block);
//        }
//    }
}

fn inject_any_method(builder: FnHeaderBuilder, attrs: &Vec<Attribute>, sig: &mut MethodSig, block: &mut Block) {
    inject_any_fn(builder, attrs, &sig.constness, &sig.ident, &mut sig.decl, block)
}

fn inject_any_fn(builder: FnHeaderBuilder, attrs: &Vec<Attribute>, constness: &Option<Const>, fn_name: &Ident,
                 fn_decl: &mut FnDecl, block: &mut Block) {
    if constness.is_some() || fn_decl.variadic.is_some() || is_not_mockable(attrs) {
        return
    }
    unignore_fn_args(&mut fn_decl.inputs);
    let header_stmt = builder.build(fn_name, &fn_decl.inputs);
    block.stmts.insert(0, header_stmt);
}

fn unignore_fn_args(inputs: &mut Punctuated<FnArg, Comma>) {
    for (i, fn_arg) in inputs.iter_mut().enumerate() {
        if let FnArg::Captured(
            ArgCaptured {
                pat: ref mut pat @ Pat::Wild(_),
                ..
            }
        ) = *fn_arg {
            *pat = Pat::Ident(
                PatIdent {
                    by_ref: None,
                    mutability: None,
                    ident: Ident::from(format!("__mocktopus_unignored_argument_{}__", i)),
                    subpat: None,
                }
            )
        }
    }
}

const INJECTOR_STOPPER_ATTRS: [&str; 2] = ["mockable", "not_mockable"];

fn is_not_mockable(attrs: &Vec<Attribute>) -> bool {
    attrs.iter()
        .filter_map(|a| a.path.segments.last())
        .map(|s| s.value().ident.as_ref())
        .any(|i| INJECTOR_STOPPER_ATTRS.contains(&i))
}
