use std::fs;
use std::path::{Path, PathBuf};

use netweft::adapter::{AdapterContext, Capability};
use netweft::adapters::builtin_registry;
use netweft::config::load::ConfigLoader;
use netweft::paths::NetweftPaths;
use netweft::resolve::ResolvedPlan;

fn fixture_config() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/proxy-services/config")
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
fn registry_exposes_nginx_reverse_proxy_capability() {
    let registry = builtin_registry().unwrap();
    let metadata = registry.get("nginx").unwrap().metadata();
    assert_eq!(metadata.id.as_str(), "nginx");
    assert!(metadata.capabilities.contains(&Capability::ReverseProxy));
    assert!(
        metadata
            .capabilities
            .contains(&Capability::CertificateIntent)
    );
}

#[test]
fn renders_stable_nginx_configuration_from_resolved_plan() {
    let paths = test_paths("nginx-render");
    let bundle = ConfigLoader::new(&paths.config_dir).load(None).unwrap();
    bundle.validate().unwrap();
    let registry = builtin_registry().unwrap();
    let plan = ResolvedPlan::new(&bundle, &paths);
    let context = AdapterContext::new(&plan).for_host("nexus");

    let output = registry.get("nginx").unwrap().render(&context).unwrap();
    assert_eq!(output.target_host.as_deref(), Some("nexus"));
    assert_eq!(
        output
            .artifacts
            .iter()
            .map(|artifact| artifact.relative_path.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        vec![
            "conf.d/dsm.suhail.ink.conf",
            "conf.d/jellyfin.suhail.ink.conf",
            "manifest.txt",
            "nginx.conf",
        ]
    );

    assert_fixture(&output.root.join("nginx.conf"), "nginx.conf");
    assert_fixture(
        &output.root.join("conf.d/dsm.suhail.ink.conf"),
        "conf.d/dsm.suhail.ink.conf",
    );
    assert_fixture(
        &output.root.join("conf.d/jellyfin.suhail.ink.conf"),
        "conf.d/jellyfin.suhail.ink.conf",
    );
    assert_fixture(&output.root.join("manifest.txt"), "manifest.txt");
}

fn assert_fixture(actual: &Path, relative: &str) {
    let expected = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/proxy-services/expected/nginx")
        .join(relative);
    assert_eq!(
        fs::read_to_string(actual).unwrap(),
        fs::read_to_string(expected).unwrap(),
        "generated output differed for {relative}"
    );
}

#[test]
fn rejects_tls_rendering_without_a_certificate_reference() {
    let config =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/shane-xfinity/config");
    let mut paths = test_paths("nginx-missing-certificate");
    paths.config_dir = config.clone();
    let bundle = ConfigLoader::new(&config).load(None).unwrap();
    let registry = builtin_registry().unwrap();
    let plan = ResolvedPlan::new(&bundle, &paths);
    let context = AdapterContext::new(&plan).for_host("nexus");

    let error = registry
        .get("nginx")
        .unwrap()
        .validate(&context)
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("has no certificate reference for the Nginx adapter")
    );
}
