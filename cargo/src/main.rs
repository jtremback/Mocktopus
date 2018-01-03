extern crate cargo;
extern crate cargo_edit;
extern crate cargo_metadata;
extern crate copy_dir;
extern crate quote;
extern crate serde_json;
extern crate syn;
extern crate toml;

use cargo::core::{Dependency, Manifest, Package, Summary, Workspace};
use cargo::ops::{self, OutputMetadataOptions};
use cargo::sources::PathSource;
use cargo::util::config::Config;
use cargo::util::important_paths;
use cargo_edit::{Dependency as DependencyEdit, Manifest as ManifestEdit};
use cargo_metadata::Metadata;
use quote::{Tokens, ToTokens};
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use syn::{Crate, ItemKind};

fn main() {
    let config = Config::default().unwrap();
    let manifest_path = important_paths::find_project_manifest(config.cwd(), "Cargo.toml").unwrap();
    let workspace = Workspace::new(&manifest_path, &config).unwrap();

    let metadata = load_metadata(&workspace);
//    for package in &metadata.packages {
//        println!("{:<43} {:<16} {}", package.name, package.version, package.manifest_path);
//    }
    let mocktopus_version = metadata.packages.iter()
        .find(|p| p.name == "mocktopus")
        .map(|p| p.version.clone());

    let mock_dir = workspace.root().join(".mocktopus");
    if mock_dir.exists() {
        fs::remove_dir_all(&mock_dir).unwrap();
    }
    fs::create_dir(&mock_dir).unwrap();

    let workspace_name = workspace.current().unwrap().name();
    for dep_package in &metadata.packages {
        if dep_package.name == workspace_name {
            println!("SKIPPING {}", workspace_name);
            continue;
        }
        println!("DEP {} {}", dep_package.name, dep_package.version);
        let target_dep_dir = mock_dir.join(format!("{} {}", dep_package.name, dep_package.version));
        let source_dep_dir = Path::new(&dep_package.manifest_path).parent().unwrap();
        println!("Copy from {:?} to {:?}", source_dep_dir, target_dep_dir);
        copy_dir::copy_dir(&source_dep_dir, &target_dep_dir).unwrap();
    }



    let package = workspace.current().unwrap();
    println!("SELF {}", package.name());
    for member in workspace.members() {
        println!("MEMBER {}", member.name())
    }
    for target in package.targets() {
//        inject_crate_root(target.src_path())
    }


    let target_roots: Vec<_> = package.targets().iter().map(|t| t.src_path()).collect();
    let config = workspace.config();
    let mut path_source = PathSource::new(package.root(), package.package_id().source_id(), config);
    let package_root = package.root();
    println!("ROOT: {:?}", package_root);
    for source_path in path_source.list_files(&package).unwrap() {
        let target_path = package_root
            .join(".mtest")
            .join(source_path.strip_prefix(package_root).unwrap());
        println!("SOURCE: {:?}\nTARGET: {:?}", source_path, target_path);
        fs::create_dir_all(target_path.parent().unwrap()).unwrap();
        if target_roots.contains(&&*source_path) {
            println!("IS ROOT\n");
            inject_root_file(&source_path, &target_path)
        } else if source_path == package.manifest_path() {
            inject_manifest_file(&source_path, &target_path)
        } else {
            fs::copy(source_path, &target_path).unwrap();
        };
    }

//    ;
}

fn load_metadata(workspace: &Workspace) -> Metadata {
    let output_metadata_ops = OutputMetadataOptions {
        features: vec![],
        no_default_features: false,
        all_features: false,
        no_deps: false,
        version: 1,
    };
    let export_info = ops::output_metadata(workspace, &output_metadata_ops).unwrap();
    let metadata_json = serde_json::to_string(&export_info).unwrap();
    serde_json::from_str(&metadata_json).unwrap()
}

fn inject_manifest_file<P: AsRef<Path>>(input_path: &Path, output_path: P) {
    let mut manifest_edit = ManifestEdit::open(&Some(input_path.to_path_buf())).unwrap();
    let table_path = ["dependencies".to_string()];
    let mocktopus_dep = DependencyEdit::new("mocktopus").set_version("=0.1.1"); // Get version from root crate deps?
    manifest_edit.insert_into_table(&table_path, &mocktopus_dep).unwrap();
    manifest_edit.write_to_file(&mut ::std::fs::File::create(output_path).unwrap()).unwrap();
}

fn inject_root_manifest_file<P: AsRef<Path>>(input_path: &Path, output_path: P) {
    let mut manifest_edit = ManifestEdit::open(&Some(input_path.to_path_buf())).unwrap();
    let table_path = ["dependencies".to_string()];
    let mocktopus_dep = DependencyEdit::new("mocktopus").set_version("=0.1.1"); // Get version from root crate deps?
    manifest_edit.insert_into_table(&table_path, &mocktopus_dep).unwrap();
    manifest_edit.write_to_file(&mut ::std::fs::File::create(output_path).unwrap()).unwrap();
}

fn inject_root_file(input_path: &Path, output_path: &Path) {
    let mut input_file = File::open(input_path).unwrap();
    let mut input_crate_string = String::new();
    input_file.read_to_string(&mut input_crate_string).unwrap();
    let output_crate_string = inject_root_string(&input_crate_string);
    let mut output_file = File::create(output_path).unwrap();
    output_file.write_all(output_crate_string.as_bytes()).unwrap();
}

fn inject_root_string(in_string: &str) -> String {
    let mut krate = syn::parse_crate(&in_string).unwrap();
    inject_root_crate(&mut krate);
    let mut tokens = Tokens::new();
    krate.to_tokens(&mut tokens);
    tokens.into_string()
}

fn inject_root_crate(krate: &mut Crate) {
    // duplicates don't matter
    let proc_macro_attr = syn::parse_inner_attr("#![feature(proc_macro)]").unwrap();
    krate.attrs.insert(0, proc_macro_attr);

    // if defined anywhere, then remove them
    let extern_crate_item = syn::parse_item("extern crate mocktopus as mocktopus_injected_by_mtest;").unwrap();
    krate.items.insert(0, extern_crate_item);

    // annotate all items
    let mockable_attr = syn::parse_outer_attr("#[mockable]").unwrap();
    krate.items.iter_mut()
        .for_each(|i| i.attrs.push(mockable_attr.clone()));
}
