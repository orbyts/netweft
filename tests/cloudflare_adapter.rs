use netweft::adapter::{AdapterContext, Capability};
use netweft::adapters::builtin_registry;
use netweft::config::load::ConfigLoader;
use netweft::paths::NetweftPaths;
use netweft::resolve::ResolvedPlan;
use std::fs;
use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/shane-xfinity/config")
}
#[test]
fn registry_exposes_cloudflare_ingress() {
    let metadata = builtin_registry()
        .unwrap()
        .get("cloudflare")
        .unwrap()
        .metadata();
    assert!(
        metadata
            .capabilities
            .contains(&Capability::CloudflareTunnel)
    );
    assert!(metadata.capabilities.contains(&Capability::CloudflareDns));
}
#[test]
fn renders_location_selected_tunnel_without_secrets() {
    let config = fixture();
    let bundle = ConfigLoader::new(&config).load(None).unwrap();
    let root = std::env::temp_dir().join(format!("netweft-cloudflare-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let paths = NetweftPaths {
        config_dir: config,
        generated_dir: root.join("generated"),
        state_dir: root.join("state"),
        cache_dir: root.join("cache"),
    };
    let resolved = ResolvedPlan::new(&bundle, &paths);
    let output = builtin_registry()
        .unwrap()
        .get("cloudflare")
        .unwrap()
        .render(&AdapterContext::new(&resolved))
        .unwrap();
    let action = fs::read_to_string(output.root.join("action-plan.txt")).unwrap();
    let apply = fs::read_to_string(output.root.join("apply-cloudflare.sh")).unwrap();
    assert!(action.contains("Direct public IPv4: unused"));
    assert!(action.contains("Apex records: untouched"));
    assert!(apply.contains("CLOUDFLARE_API_TOKEN_SUHAIL_INK_TUNNEL"));
    assert!(apply.contains("CLOUDFLARE_API_TOKEN_SUHAIL_INK_DNS"));
    assert!(apply.contains("call_tunnel"));
    assert!(apply.contains("call_dns"));
    assert!(!apply.contains("Bearer ey"));
}
