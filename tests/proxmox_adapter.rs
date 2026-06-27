use std::fs;
use std::path::{Path, PathBuf};

use netweft::adapter::{AdapterContext, Capability};
use netweft::adapters::builtin_registry;
use netweft::config::load::ConfigLoader;
use netweft::paths::NetweftPaths;
use netweft::resolve::ResolvedPlan;

fn fixture_config() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/host-network/config")
}

fn expected_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/host-network/expected/proxmox")
}

fn test_paths(name: &str) -> NetweftPaths {
    let root = std::env::temp_dir().join(format!("netweft-{name}-{}", std::process::id()));

    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }

    NetweftPaths {
        config_dir: fixture_config(),
        generated_dir: root.join("generated"),
        state_dir: root.join("state"),
        cache_dir: root.join("cache"),
    }
}

#[test]
fn registry_exposes_proxmox_host_networking() {
    let registry = builtin_registry().unwrap();
    let metadata = registry.get("proxmox").unwrap().metadata();

    assert_eq!(metadata.id.as_str(), "proxmox");
    assert!(metadata.capabilities.contains(&Capability::HostNetworking));
}

#[test]
fn renders_stable_proxmox_configuration() {
    let paths = test_paths("proxmox-render");

    let bundle = ConfigLoader::new(&paths.config_dir).load(None).unwrap();

    bundle.validate().unwrap();

    let registry = builtin_registry().unwrap();
    let plan = ResolvedPlan::new(&bundle, &paths);
    let context = AdapterContext::new(&plan).for_host("zion");

    let output = registry.get("proxmox").unwrap().render(&context).unwrap();

    assert_eq!(output.target_host.as_deref(), Some("zion"));

    assert_eq!(
        output
            .artifacts
            .iter()
            .map(|artifact| { artifact.relative_path.to_string_lossy().into_owned() })
            .collect::<Vec<_>>(),
        vec![
            "etc/hosts",
            "etc/network/interfaces",
            "etc/resolv.conf",
            "manifest.txt",
        ]
    );

    assert_fixture(
        &output.root.join("etc/network/interfaces"),
        "etc/network/interfaces",
    );

    assert_fixture(&output.root.join("etc/hosts"), "etc/hosts");

    assert_fixture(&output.root.join("etc/resolv.conf"), "etc/resolv.conf");

    assert_fixture(&output.root.join("manifest.txt"), "manifest.txt");
}

#[test]
fn requires_an_explicit_host() {
    let paths = test_paths("proxmox-no-host");

    let bundle = ConfigLoader::new(&paths.config_dir).load(None).unwrap();

    let registry = builtin_registry().unwrap();
    let plan = ResolvedPlan::new(&bundle, &paths);
    let context = AdapterContext::new(&plan);

    let error = registry
        .get("proxmox")
        .unwrap()
        .validate(&context)
        .unwrap_err()
        .to_string();

    assert!(error.contains("requires --host"), "{error}");
}

#[test]
fn rejects_a_non_proxmox_host() {
    let paths = test_paths("proxmox-wrong-host");

    let bundle = ConfigLoader::new(&paths.config_dir).load(None).unwrap();

    let registry = builtin_registry().unwrap();
    let plan = ResolvedPlan::new(&bundle, &paths);

    let error = registry
        .get("proxmox")
        .unwrap()
        .validate(&AdapterContext::new(&plan).for_host("quasar"))
        .unwrap_err()
        .to_string();

    assert!(error.contains("has no host network profile"), "{error}");
}

fn assert_fixture(actual: &Path, relative: &str) {
    let expected = expected_root().join(relative);

    assert_eq!(
        fs::read_to_string(actual).unwrap(),
        fs::read_to_string(expected).unwrap(),
        "generated output differed for {relative}"
    );
}
