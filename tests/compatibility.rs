use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use netweft::config::load::ConfigLoader;
use netweft::paths::NetweftPaths;
use netweft::plan::dns::{ResolvedRecord, resolve_dns_plan};
use netweft::plan::env::resolve_env_plan;
use netweft::render::bind::render_bind;

fn fixture_config() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/shane-xfinity/config")
}

fn temp_root(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("netweft-{name}-{}-{nonce}", std::process::id()))
}

#[test]
fn current_configuration_preserves_resolved_behavior() {
    let config = fixture_config();
    let bundle = ConfigLoader::new(&config).load(None).unwrap();
    let report = bundle.validate().unwrap();

    assert_eq!(bundle.location.name, "shane-xfinity");
    assert_eq!(bundle.inventory.hosts.len(), 8);
    assert_eq!(bundle.networks.networks.len(), 2);
    assert_eq!(bundle.services.services.len(), 5);
    assert_eq!(bundle.dns.zones.len(), 4);
    assert!(report.warnings.len() <= 8);

    let dns = resolve_dns_plan(&bundle).unwrap();
    assert_eq!(dns.container_ipv4.to_string(), "10.78.29.29");
    assert_eq!(dns.ingress_ipv4.to_string(), "10.214.90.10");

    let primary = dns
        .zones
        .iter()
        .find(|zone| zone.name == "suhail.ink")
        .unwrap();

    assert!(primary.records.contains(&ResolvedRecord::A {
        name: "ds1621plus.suhail.ink.".to_owned(),
        address: "10.214.90.20".parse().unwrap(),
    }));
    let life = dns
        .zones
        .iter()
        .find(|zone| zone.name == "suhail.life")
        .unwrap();

    assert!(life.records.contains(&ResolvedRecord::A {
        name: "family.suhail.life.".to_owned(),
        address: "10.214.90.10".parse().unwrap(),
    }));
}

#[test]
fn current_bind_output_keeps_expected_files_and_records() {
    let config = fixture_config();
    let bundle = ConfigLoader::new(&config).load(None).unwrap();
    bundle.validate().unwrap();
    let plan = resolve_dns_plan(&bundle).unwrap();

    let root = temp_root("bind-compat");
    let output = root.join("bind");
    render_bind(&plan, &output).unwrap();

    for relative in [
        "named.conf",
        "named.conf.options",
        "named.conf.local",
        "manifest.txt",
        "zones/db.suhail.ink",
        "zones/db.90.214.10.in-addr.arpa",
    ] {
        assert!(output.join(relative).is_file(), "missing {relative}");
    }

    let zone = fs::read_to_string(output.join("zones/db.suhail.ink")).unwrap();
    assert!(zone.contains("ds1621plus"));
    assert!(zone.contains("10.214.90.20"));
    assert!(zone.contains("nginx"));
    assert!(zone.contains("10.214.90.10"));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn current_nexus_environment_keeps_expected_addresses() {
    let config = fixture_config();
    let bundle = ConfigLoader::new(&config).load(None).unwrap();
    bundle.validate().unwrap();

    let root = temp_root("env-compat");
    let paths = NetweftPaths {
        config_dir: config,
        generated_dir: root.join("generated"),
        state_dir: root.join("state"),
        cache_dir: root.join("cache"),
    };

    let plan = resolve_env_plan(&bundle, &paths, "nexus").unwrap();
    assert_eq!(plan.variables["NEXUS_LAN_IPV4"], "10.214.90.10");
    assert_eq!(plan.variables["BIND9_IPV4"], "10.78.29.29");
    assert_eq!(plan.variables["NGINX_IPV4"], "10.78.88.88");
    assert_eq!(plan.variables["DB_IPV4"], "10.78.29.31");
    assert!(plan.variables["TS_ADVERTISE_ROUTES"].contains("10.214.90.0/24"));

    if Path::new(&root).exists() {
        fs::remove_dir_all(root).unwrap();
    }
}
