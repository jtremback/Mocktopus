extern crate cargo;
extern crate quote;
extern crate syn;
extern crate toml;

use cargo::core::{Dependency, Manifest, Package, Summary, Workspace};
use cargo::util::config::Config;
use cargo::util::important_paths;
use quote::{Tokens, ToTokens};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::rc::Rc;
use syn::{Crate, ItemKind};

fn main() {
    let config = Config::default().unwrap();
    let manifest_path = important_paths::find_project_manifest(config.cwd(), "Cargo.toml").unwrap();
    let workspace = Workspace::new(&manifest_path, &config).unwrap();
    let package = workspace.current().unwrap();
    for target in package.targets() {
//        inject_file(target.src_path())
    }
    let mut deps = package.dependencies().to_vec();
    for dep in &deps {
        println!("DEP {}", dep.name());
    }
//    for dep in &deps {
//        println!("DEP {:#?}", dep);
//    }

    println!("\nOLD MANIFEST {:?}:\n{}",
             package.manifest_path(), toml::to_string(package.manifest().original()).unwrap());
    let new_package = clone_package_with_deps(package, deps);
    let mut deps = package.dependencies().to_vec();
    println!("\nNEW MANIFEST {:?}:\n{}",
             new_package.manifest_path(), toml::to_string(new_package.manifest().original()).unwrap());
//    println!("{}", package.to_registry_toml());
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

fn clone_package_with_deps(package: &Package, new_deps: Vec<Dependency>) -> Package {
    let manifest = package.manifest();
    let summary = manifest.summary();
    let new_summary = Summary::new(
        summary.package_id().clone(),
        new_deps,
        summary.features().clone()
    ).unwrap();
    let new_manifest = Manifest::new(
        new_summary,
        manifest.targets().to_vec(),
        manifest.exclude().to_vec(),
        manifest.include().to_vec(),
        manifest.links().map(|s| s.to_string()),
        manifest.metadata().clone(),
        manifest.profiles().clone(),
        manifest.publish(),
        manifest.replace().to_vec(),
        manifest.patch().clone(),
        manifest.workspace_config().clone(),
        manifest.features().clone(),
        None,
        Rc::new(manifest.original().prepare_for_publish()),
    );
    Package::new(new_manifest, package.manifest_path().clone())
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
    println!("\n{:?} AFTER INJECTION:\n{}", path, out_file_content);
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

    // if defined anywhere, then remove them
    let extern_crate_item = syn::parse_item("extern crate mocktopus;").unwrap();
    krate.items.insert(0, extern_crate_item);

    // annotate all mods, if defined, don't insert
    let mockable_attr = syn::parse_outer_attr("#[mockable]").unwrap();
    for item in &mut krate.items {
        if let ItemKind::Mod(_) = item.node {
            item.attrs.push(mockable_attr.clone())
        }
    }
}
