extern crate cargo;
extern crate quote;
extern crate syn;

use cargo::core::Workspace;
use cargo::util::config::Config;
use cargo::util::important_paths;
use quote::{Tokens, ToTokens};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use syn::{Crate, ItemKind};

fn main() {
    let config = Config::default().unwrap();
    let manifest_path = important_paths::find_project_manifest(config.cwd(), "Cargo.toml").unwrap();
    let workspace = Workspace::new(&manifest_path, &config).unwrap();
    let package = workspace.current().unwrap();
    for target in package.targets() {
        println!("\nSelf: {:?}", target.src_path());
        inject_file(target.src_path())
    }
    //    for dependency in package.dependencies() {
    //        println!("\nExternal: {:#?}", dependency);
    //    }
    //    let packages_set = ops::fetch(&workspace).unwrap().1;
    //    for package_id in packages_set.package_ids() {
    //        println!("\nPackage: {:?} ", package_id.name());
    //        for target in packages_set.get(package_id).unwrap().targets() {
    //            println!("Target: {:?}", target.src_path())
    //        }
    //    }
}

fn inject_file(path: &Path) {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .unwrap();
    let mut in_file_content = String::new();
    file.read_to_string(&mut in_file_content).unwrap();
    let out_file_content = inject_string(&in_file_content);
    file.seek(SeekFrom::Start(0)).unwrap();
    //    file.write(out_file_content.as_bytes()).unwrap();
    println!("NEW_ONE:\n{}", out_file_content);
}

fn inject_string(in_string: &str) -> String {
    let mut krate = syn::parse_crate(&in_string).unwrap();
    inject_crate(&mut krate);
    let mut tokens = Tokens::new();
    krate.to_tokens(&mut tokens);
    tokens.into_string()
}

fn inject_crate(krate: &mut Crate) {
    // duplicates don't matter
    let proc_macro_attr = syn::parse_inner_attr("#![feature(proc_macro)]").unwrap();
    krate.attrs.insert(0, proc_macro_attr);

    // if defined anywhere, then don't insert
    let extern_crate_item = syn::parse_item("extern crate mocktopus;").unwrap();
    krate.items.insert(0, extern_crate_item);

    // annotate all mods, if defined, don't insert
    let mockable_attr = syn::parse_outer_attr("#[mockable]").unwrap();
    for item in &mut krate.items {
        if let ItemKind::Mod(_) = item.node {
            item.attrs.push(mockable_attr.clone())
        }
    }

    for item in &krate.items {
        println!("{:?}", item);
    }
}
