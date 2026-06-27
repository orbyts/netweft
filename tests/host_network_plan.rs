use std::path::PathBuf;

use netweft::config::load::ConfigLoader;
use netweft::model::HostNetworkProvider;
use netweft::plan::host_network::{ResolvedHostIpv6Mode, resolve_host_network_plan};

fn fixture_config() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/host-network/config")
}

#[test]
fn resolves_zion_host_network_plan() {
    let bundle = ConfigLoader::new(&fixture_config()).load(None).unwrap();

    let plan = resolve_host_network_plan(&bundle, "zion").unwrap();

    assert_eq!(plan.location, "shane-xfinity");
    assert_eq!(plan.host, "zion");
    assert_eq!(plan.fqdn, "zion.suhail.ink");
    assert_eq!(plan.provider, HostNetworkProvider::ProxmoxIfupdown2);
    assert_eq!(plan.management_interface, "lan");
    assert!(plan.preserve_includes);
    assert_eq!(plan.dns_servers.len(), 1);
    assert_eq!(plan.dns_servers[0].to_string(), "10.214.90.10");
    assert_eq!(plan.search_domains, vec!["suhail.ink".to_owned()]);

    assert_eq!(plan.links.len(), 5);
    assert_eq!(plan.bridges.len(), 2);

    let vmbr0 = plan
        .bridges
        .iter()
        .find(|bridge| bridge.name == "vmbr0")
        .unwrap();

    assert_eq!(vmbr0.ports, vec!["enp3s0f1np1"]);
    assert_eq!(vmbr0.location_interface.as_deref(), Some("lan"));
    assert_eq!(vmbr0.segment.as_deref(), Some("main"));
    assert_eq!(vmbr0.ipv4.unwrap().to_string(), "10.214.90.30/24");
    assert_eq!(vmbr0.ipv4_gateway.unwrap().to_string(), "10.214.90.1");
    assert_eq!(vmbr0.ipv6_mode, ResolvedHostIpv6Mode::Slaac);
    assert!(vmbr0.vlan_aware);
    assert_eq!(vmbr0.allowed_vlans.as_deref(), Some("2-4094"));
    assert!(!vmbr0.stp);
    assert_eq!(vmbr0.forward_delay, 2);
    assert_eq!(vmbr0.comment.as_deref(), Some("Primary"));

    let vmbr1 = plan
        .bridges
        .iter()
        .find(|bridge| bridge.name == "vmbr1")
        .unwrap();

    assert!(vmbr1.ports.is_empty());
    assert!(vmbr1.location_interface.is_none());
    assert!(vmbr1.ipv4.is_none());
    assert!(vmbr1.ipv4_gateway.is_none());
}

#[test]
fn rejects_a_host_without_a_network_profile() {
    let bundle = ConfigLoader::new(&fixture_config()).load(None).unwrap();

    let error = resolve_host_network_plan(&bundle, "quasar")
        .unwrap_err()
        .to_string();

    assert!(error.contains("has no host network profile"), "{error}");
}

#[test]
fn rejects_an_unknown_host() {
    let bundle = ConfigLoader::new(&fixture_config()).load(None).unwrap();

    let error = resolve_host_network_plan(&bundle, "nonexistent")
        .unwrap_err()
        .to_string();

    assert!(error.contains("unknown host"), "{error}");
}
