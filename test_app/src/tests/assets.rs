//! Shared runtime-model format and dependency contract for the game and zoo.
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

fn files_under(directory: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files.extend(files_under(&path));
        } else {
            files.push(path);
        }
    }
    files
}

fn dependency(model: &Path, uri: &str, root: &Path, extension: &str) -> PathBuf {
    assert!(
        !uri.starts_with("data:") && !uri.contains("://"),
        "{model:?}: external local files only: {uri}"
    );
    // glTF image names may be percent-encoded by Blender.
    let mut decoded = Vec::new();
    let mut bytes = uri.as_bytes().iter().copied();
    while let Some(byte) = bytes.next() {
        decoded.push(if byte == b'%' {
            let hex = [bytes.next().unwrap(), bytes.next().unwrap()];
            u8::from_str_radix(std::str::from_utf8(&hex).unwrap(), 16).unwrap()
        } else {
            byte
        });
    }
    let name = String::from_utf8(decoded).unwrap();
    let relative = Path::new(&name);
    assert!(
        relative.is_relative(),
        "{model:?}: absolute resource path {uri}"
    );
    let path = model.parent().unwrap().join(relative);
    assert!(path.is_file(), "{model:?}: missing resource {uri}");
    let path = path.canonicalize().unwrap();
    assert!(
        path.starts_with(root),
        "{model:?}: resource outside runtime asset root: {uri}"
    );
    assert_eq!(
        path.extension().and_then(|v| v.to_str()),
        Some(extension),
        "{model:?}: unexpected resource format {uri}"
    );
    path
}

#[test]
fn runtime_models_use_only_external_gltf_bin_and_png() {
    use bevy::gltf::gltf::{self, buffer, image};
    let root = Path::new(crate::ASSET_DIR).canonicalize().unwrap();
    let files = files_under(&root);
    let forbidden: Vec<_> = files
        .iter()
        .filter(|path| {
            let extension = path
                .extension()
                .and_then(|v| v.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            matches!(extension.as_str(), "glb" | "fbx" | "obj" | "mtl")
                || extension.starts_with("blend")
        })
        .collect();
    assert!(
        forbidden.is_empty(),
        "non-runtime or parallel model formats ({} total): {:?}",
        forbidden.len(),
        forbidden.iter().take(8).collect::<Vec<_>>()
    );
    let mut models = 0;
    let mut referenced_buffers = BTreeSet::new();
    for path in files
        .iter()
        .filter(|p| p.extension().is_some_and(|e| e == "gltf"))
    {
        let document = gltf::Gltf::from_slice(&fs::read(path).unwrap())
            .unwrap_or_else(|error| panic!("invalid glTF {path:?}: {error}"));
        assert!(document.blob.is_none());
        for buffer in document.buffers() {
            let buffer::Source::Uri(uri) = buffer.source() else {
                panic!("{path:?}: buffer must be an external .bin file");
            };
            let external = dependency(path, uri, &root, "bin");
            assert!(
                fs::metadata(&external).unwrap().len() >= buffer.length() as u64,
                "{path:?}: truncated buffer {uri}"
            );
            referenced_buffers.insert(external);
        }
        for image in document.images() {
            let image::Source::Uri { uri, .. } = image.source() else {
                panic!("{path:?}: textures must be external PNGs");
            };
            let external = dependency(path, uri, &root, "png");
            assert!(
                fs::read(external)
                    .unwrap()
                    .starts_with(b"\x89PNG\r\n\x1a\n")
            );
        }
        models += 1;
    }
    assert!(models > 0);
    for path in files
        .iter()
        .filter(|p| p.extension().is_some_and(|e| e == "bin"))
    {
        assert!(
            referenced_buffers.contains(&path.canonicalize().unwrap()),
            "orphaned model buffer {path:?}"
        );
    }
    for model in [
        "Assets/gltf/Dummy_Base.gltf",
        "Assets/gltf/target.gltf",
        "custom/courtyard/courtyard.gltf",
        "custom/training_blaster.gltf",
    ] {
        assert!(root.join(model).is_file(), "missing runtime model {model}");
    }
}

#[test]
fn game_spawn_paths_point_to_existing_gltf_files() {
    use bevy::{prelude::*, world_serialization::WorldAsset};
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<WorldAsset>()
        .init_asset::<Mesh>()
        .init_asset::<StandardMaterial>()
        .init_resource::<crate::level::LevelAssets>()
        .init_resource::<crate::settings::Settings>();
    let world = app.world_mut();
    world.run_system_cached(crate::level::spawn_level).unwrap();
    world
        .run_system_cached(crate::player::spawn_player)
        .unwrap();
    world
        .run_system_cached(crate::items::coins::spawn_coins)
        .unwrap();
    world
        .run_system_cached(crate::practice::spawn_range)
        .unwrap();
    let tracked = &world.resource::<crate::level::LevelAssets>().0;
    assert!(!tracked.is_empty());
    for handle in tracked {
        let path = handle.path().unwrap().path();
        assert_eq!(path.extension().and_then(|v| v.to_str()), Some("gltf"));
        assert!(
            Path::new(crate::ASSET_DIR).join(path).is_file(),
            "missing spawned model {path:?}"
        );
    }
}

#[test]
fn kaykit_models_share_one_texture_file() {
    let root = Path::new(crate::ASSET_DIR);
    let textures: Vec<_> = files_under(root)
        .into_iter()
        .filter(|p| {
            p.file_name()
                .is_some_and(|name| name == "prototypebits_texture.png")
        })
        .collect();
    assert_eq!(
        textures,
        [root.join("Assets/gltf/prototypebits_texture.png")]
    );
    for entry in fs::read_dir(root.join("Assets/gltf")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "gltf") {
            let document: serde_json::Value =
                serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
            for image in document["images"].as_array().unwrap() {
                assert_eq!(image["uri"], "prototypebits_texture.png");
            }
        }
    }
}
