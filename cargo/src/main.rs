extern crate cargo;
extern crate cargo_edit;
extern crate quote;
extern crate syn;
extern crate toml;

use cargo::core::{Dependency, Manifest, Package, Summary, Workspace};
use cargo::sources::PathSource;
use cargo::util::config::Config;
use cargo::util::important_paths;
use cargo_edit::{Dependency as DependencyEdit, Manifest as ManifestEdit};
use quote::{Tokens, ToTokens};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use syn::{Crate, ItemKind};

fn main() {
    let config = Config::default().unwrap();
    let manifest_path = important_paths::find_project_manifest(config.cwd(), "Cargo.toml").unwrap();
    let workspace = Workspace::new(&manifest_path, &config).unwrap();
    let package = workspace.current().unwrap();
    println!("SELF {}", package.name());
    for member in workspace.members() {
        println!("MEMBER {}", member.name())
    }
    for target in package.targets() {
//        inject_crate_root(target.src_path())
    }


    let config = workspace.config();
    let mut path_source = PathSource::new(package.root(), package.package_id().source_id(), config);
    let root = package.root();
    println!("ROOT: {:?}", root);
    for source_path in path_source.list_files(&package).unwrap() {
        let target_path = root.join(".mocktopus").join(source_path.strip_prefix(root).unwrap());
        println!("{:?} =>\n{:?}\n", source_path, target_path);
    }

//    inject_manifest(package.manifest_path(), "modified.toml");
}

fn inject_manifest<P: AsRef<Path>>(input_path: &Path, output_path: P) {
    let mut manifest_edit = ManifestEdit::open(&Some(input_path.to_path_buf())).unwrap();
    let table_path = ["dependencies".to_string()];
    let mocktopus_dep = DependencyEdit::new("mocktopus").set_version("=0.1.1");
    manifest_edit.insert_into_table(&table_path, &mocktopus_dep).unwrap();
    manifest_edit.write_to_file(&mut ::std::fs::File::create(output_path).unwrap()).unwrap();
}

fn inject_crate_root<P: AsRef<Path>>(input_path: &Path, output_path: P) {
    let mut input_file = File::open(input_path).unwrap();
    let mut input_crate_string = String::new();
    input_file.read_to_string(&mut input_crate_string).unwrap();
    let output_crate_string = inject_crate_root_string(&input_crate_string);
    let mut output_file = File::create(output_path).unwrap();
    output_file.write_all(output_crate_string.as_bytes()).unwrap();
}

fn inject_crate_root_string(in_string: &str) -> String {
    let mut krate = syn::parse_crate(&in_string).unwrap();
    inject_crate_root_crate(&mut krate);
    let mut tokens = Tokens::new();
    krate.to_tokens(&mut tokens);
    tokens.into_string()
}

fn inject_crate_root_crate(krate: &mut Crate) {
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
