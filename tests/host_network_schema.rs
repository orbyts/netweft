use netweft::model::{HostNetworkLinkKind, HostNetworkProvider, InventoryFile};

#[test]
fn existing_hosts_do_not_require_a_network_profile() {
    let source = r#"
schema_version = 1

[domains]
primary = "suhail.ink"
additional = []

[hosts.quasar]
kind = "workstation"
roles = ["client", "development"]
ssh_user = "suhail"
enabled = true
"#;

    let inventory: InventoryFile = toml::from_str(source).unwrap();
    let quasar = inventory.hosts.get("quasar").unwrap();

    assert!(quasar.network.is_none());
    assert_eq!(quasar.ssh_user.as_deref(), Some("suhail"));
    assert!(quasar.enabled);
}

#[test]
fn parses_a_proxmox_host_network_profile() {
    let source = r#"
schema_version = 1

[domains]
primary = "suhail.ink"
additional = []

[hosts.zion]
kind = "physical"
roles = ["proxmox"]
ssh_user = "root"
enabled = true

[hosts.zion.network]
provider = "proxmox-ifupdown2"
management_interface = "lan"
preserve_includes = true

[[hosts.zion.network.links]]
name = "enp3s0f0np0"
kind = "ethernet"

[[hosts.zion.network.links]]
name = "enp3s0f1np1"
kind = "ethernet"

[[hosts.zion.network.links]]
name = "wlp92s0"
kind = "wifi"

[[hosts.zion.network.bridges]]
name = "vmbr0"
ports = ["enp3s0f1np1"]
location_interface = "lan"
vlan_aware = true
allowed_vlans = "2-4094"
stp = false
forward_delay = 0
comment = "Primary"

[[hosts.zion.network.bridges]]
name = "vmbr1"
ports = []
vlan_aware = true
allowed_vlans = "2-4094"
stp = false
forward_delay = 0
comment = "VM & CT Network"
"#;

    let inventory: InventoryFile = toml::from_str(source).unwrap();
    let zion = inventory.hosts.get("zion").unwrap();
    let network = zion.network.as_ref().unwrap();

    assert_eq!(network.provider, HostNetworkProvider::ProxmoxIfupdown2);
    assert_eq!(network.management_interface, "lan");
    assert!(network.preserve_includes);

    assert_eq!(network.links.len(), 3);
    assert_eq!(network.links[0].name, "enp3s0f0np0");
    assert_eq!(network.links[0].kind, HostNetworkLinkKind::Ethernet);
    assert_eq!(network.links[2].kind, HostNetworkLinkKind::Wifi);

    assert_eq!(network.bridges.len(), 2);

    let vmbr0 = &network.bridges[0];
    assert_eq!(vmbr0.name, "vmbr0");
    assert_eq!(vmbr0.ports, vec!["enp3s0f1np1"]);
    assert_eq!(vmbr0.location_interface.as_deref(), Some("lan"));
    assert!(vmbr0.vlan_aware);
    assert_eq!(vmbr0.allowed_vlans.as_deref(), Some("2-4094"));
    assert!(!vmbr0.stp);
    assert_eq!(vmbr0.forward_delay, 0);

    let vmbr1 = &network.bridges[1];
    assert_eq!(vmbr1.name, "vmbr1");
    assert!(vmbr1.ports.is_empty());
    assert!(vmbr1.location_interface.is_none());
}

#[test]
fn rejects_unknown_host_network_fields() {
    let source = r#"
schema_version = 1

[domains]
primary = "suhail.ink"
additional = []

[hosts.zion]
kind = "physical"
roles = ["proxmox"]
enabled = true

[hosts.zion.network]
provider = "proxmox-ifupdown2"
management_interface = "lan"
invented_field = true
"#;

    let error = toml::from_str::<InventoryFile>(source).unwrap_err();
    let message = error.to_string();

    assert!(message.contains("invented_field"));
}
