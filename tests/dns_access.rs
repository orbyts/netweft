use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use netweft::config::load::ConfigLoader;
use netweft::plan::dns_access::derive_dns_access;

#[test]
fn router_advertised_mode_derives_reachable_networks() {
    let root = fixture(
        "router-advertised",
        r#"
schema_version = 1
name = "test"

[router]
kind = "test"
managed = false
supports_vlans = false

[ipv6]
mode = "router-advertised"
prefix = "2001:db8:1::/64"
stability = "dynamic"

[segments.main]
kind = "lan"
ipv4_cidr = "192.168.10.0/24"
ipv4_gateway = "192.168.10.1"

[tailscale]
enabled = true
strategy = "subnet-router"
primary_router = "nexus"

[tailscale.routers.nexus]
advertise = ["segment:main", "network:containers"]
"#,
    );

    let bundle = ConfigLoader::new(&root).load(None).unwrap();
    bundle.validate().unwrap();
    let access = derive_dns_access(&bundle).unwrap();

    assert!(access.ipv4.contains(&"192.168.10.0/24".parse().unwrap()));
    assert!(access.ipv4.contains(&"10.78.0.0/16".parse().unwrap()));
    assert!(access.ipv4.contains(&"100.64.0.0/10".parse().unwrap()));
    assert!(access.ipv6.contains(&"2001:db8:1::/64".parse().unwrap()));
    assert!(
        access
            .ipv6
            .contains(&"fd7a:115c:a1e0::/48".parse().unwrap())
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn disabled_ipv6_adds_no_isp_prefix() {
    let root = fixture(
        "disabled",
        r#"
schema_version = 1
name = "test"

[router]
kind = "test"
managed = false
supports_vlans = false

[ipv6]
mode = "disabled"
stability = "dynamic"

[segments.main]
kind = "lan"
ipv4_cidr = "192.168.1.0/24"
ipv4_gateway = "192.168.1.1"
"#,
    );

    let bundle = ConfigLoader::new(&root).load(None).unwrap();
    bundle.validate().unwrap();
    let access = derive_dns_access(&bundle).unwrap();

    assert!(access.ipv4.contains(&"192.168.1.0/24".parse().unwrap()));
    assert_eq!(access.ipv6, vec!["::1/128".parse().unwrap()]);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn delegated_mode_adds_owned_delegation() {
    let root = fixture(
        "delegated",
        r#"
schema_version = 1
name = "test"

[router]
kind = "test"
managed = true
supports_vlans = true

[ipv6]
mode = "delegated"
prefix = "2001:db8:1200::/56"
subnet_prefix_length = 64
stability = "dynamic"

[segments.main]
kind = "lan"
ipv4_cidr = "10.0.0.0/24"
ipv4_gateway = "10.0.0.1"
"#,
    );

    let bundle = ConfigLoader::new(&root).load(None).unwrap();
    bundle.validate().unwrap();
    let access = derive_dns_access(&bundle).unwrap();

    assert!(access.ipv6.contains(&"2001:db8:1200::/56".parse().unwrap()));

    fs::remove_dir_all(root).unwrap();
}

fn fixture(name: &str, location: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("netweft-{name}-{}-{nonce}", std::process::id()));
    fs::create_dir_all(root.join("locations")).unwrap();

    write(
        &root,
        "netweft.toml",
        "schema_version = 1\nactive_location = \"test\"\n",
    );

    write(
        &root,
        "inventory.toml",
        r#"
schema_version = 1
[domains]
primary = "example.test"
additional = []
[hosts.nexus]
kind = "physical"
roles = ["dns"]
enabled = true
"#,
    );

    write(
        &root,
        "networks.toml",
        r#"
schema_version = 1
[networks.containers]
kind = "docker"
owner = "nexus"
ipv4_cidr = "10.78.0.0/16"
ipv4_gateway = "10.78.0.1"
allocation_key = "containers"
dns_clients = true
[networks.containers.routing]
from = "nexus"
mode = "direct"
"#,
    );

    write(
        &root,
        "services.toml",
        r#"
schema_version = 1
[services.bind9]
kind = "dns"
host = "nexus"
network = "containers"
enabled = true
[services.bind9.address]
ipv4 = "10.78.29.29"
"#,
    );

    write(
        &root,
        "dns.toml",
        r#"
schema_version = 1
[dns]
provider = "bind9"
service = "bind9"
default_ttl = 300
negative_ttl = 60
[dns.soa]
primary_nameserver = "ns1.example.test."
responsible_mailbox = "admin.example.test."
refresh = 3600
retry = 600
expire = 86400
[dns.recursion]
enabled = true
include_location_segments = true
include_tailscale = true
include_ula = true
include_docker_networks = true
[dns.forwarders]
ipv4 = ["1.1.1.1"]
ipv6 = []
"#,
    );

    write(&root, "allocations.toml", "schema_version = 1\n");
    write(&root, "locations/test.toml", location);

    root
}

fn write(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents.trim_start()).unwrap();
}
