use std::path::PathBuf;

use netweft::adapter::{AdapterContext, Capability};
use netweft::adapters::builtin_registry;
use netweft::config::load::ConfigLoader;
use netweft::paths::NetweftPaths;

fn fixture_config() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/shane-xfinity/config")
}

#[test]
fn builtin_registry_exposes_bind_and_environment_adapters() {
    let registry = builtin_registry().unwrap();
    let ids: Vec<_> = registry
        .iter()
        .map(|adapter| adapter.metadata().id.as_str())
        .collect();

    assert_eq!(ids, vec!["bind", "env"]);
    assert!(
        registry
            .get("bind")
            .unwrap()
            .metadata()
            .capabilities
            .contains(&Capability::AuthoritativeDns)
    );
}

#[test]
fn adapters_validate_against_current_configuration() {
    let config = fixture_config();
    let bundle = ConfigLoader::new(&config).load(None).unwrap();
    let paths = NetweftPaths {
        config_dir: config,
        generated_dir: std::env::temp_dir().join("netweft-adapter-test/generated"),
        state_dir: std::env::temp_dir().join("netweft-adapter-test/state"),
        cache_dir: std::env::temp_dir().join("netweft-adapter-test/cache"),
    };
    let registry = builtin_registry().unwrap();
    let context = AdapterContext::new(&bundle, &paths);

    registry.get("bind").unwrap().validate(&context).unwrap();
    registry
        .get("env")
        .unwrap()
        .validate(&context.for_host("nexus"))
        .unwrap();
}
