use cargo::core::{Package as CargoPackage, SourceId};
use cargo::core::manifest::EitherManifest;
use cargo::sources::PathSource;
use cargo::util::Config;
use cargo::util::toml;
use package_kind::PackageKind;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct PackageInfo {
    pub id: String,
    pub kind: PackageKind,
    pub root: PathBuf,
    pub files: Vec<PathBuf>,
    pub dep_names_to_ids: HashMap<String, String>,
}

impl PackageInfo {
    pub fn new(id: &str, manifest_path: &str, kind: PackageKind, dep_names_to_ids: HashMap<String, String>) -> Self {
        let mut root = PathBuf::from(manifest_path);
        let files = get_package_files(&root);
        if !root.pop() {
            panic!("43");
        }
        PackageInfo {
            id: id.to_string(),
            kind,
            root,
            files,
            dep_names_to_ids,
        }
    }
}

fn get_package_files(src_manifest: &PathBuf) -> Vec<PathBuf> {
    let src = src_manifest.parent().expect("34");
    let source_id = SourceId::for_path(src).expect("32");
    let config = Config::default().expect("30");
    let (either_manifest, _) = toml::read_manifest(src_manifest, &source_id, &config).expect("33");
    let manifest = match either_manifest {
        EitherManifest::Real(manifest) => manifest,
        EitherManifest::Virtual(_) => return vec![src_manifest.clone()],
    };
    PathSource::new(src_manifest, &source_id, &config)
        .list_files(&CargoPackage::new(manifest, src_manifest))
        .expect("31")
}
