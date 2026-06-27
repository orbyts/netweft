use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use netweft::config::load::ConfigLoader;

fn source_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/host-network/config")
}

fn temporary_fixture(name: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let root = std::env::temp_dir().join(format!("netweft-{name}-{}-{stamp}", std::process::id()));

    copy_tree(&source_fixture(), &root);
    root
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();

    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());

        if source_path.is_dir() {
            copy_tree(&source_path, &destination_path);
        } else {
            fs::copy(source_path, destination_path).unwrap();
        }
    }
}

fn load_and_validate(root: &Path) -> anyhow::Result<()> {
    let bundle = ConfigLoader::new(root).load(None)?;
    bundle.validate()?;
    Ok(())
}

#[test]
fn valid_zion_topology_passes_validation() {
    let root = temporary_fixture("valid");
    load_and_validate(&root).unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejects_unknown_bridge_port() {
    let root = temporary_fixture("unknown-port");
    let inventory = root.join("inventory.toml");

    let text = fs::read_to_string(&inventory)
        .unwrap()
        .replace(r#"ports = ["enp3s0f1np1"]"#, r#"ports = ["missing0"]"#);

    fs::write(&inventory, text).unwrap();

    let error = load_and_validate(&root).unwrap_err().to_string();

    assert!(error.contains("unknown link 'missing0'"), "{error}");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejects_duplicate_bridge_link_assignment() {
    let root = temporary_fixture("duplicate-link");
    let inventory = root.join("inventory.toml");

    let text = fs::read_to_string(&inventory).unwrap().replace(
        r#"name = "vmbr1"
ports = []"#,
        r#"name = "vmbr1"
ports = ["enp3s0f1np1"]"#,
    );

    fs::write(&inventory, text).unwrap();

    let error = load_and_validate(&root).unwrap_err().to_string();

    assert!(error.contains("assigned to both bridge"), "{error}");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejects_allowed_vlans_on_non_vlan_aware_bridge() {
    let root = temporary_fixture("non-vlan-aware");
    let inventory = root.join("inventory.toml");

    let text = fs::read_to_string(&inventory).unwrap().replacen(
        "vlan_aware = true",
        "vlan_aware = false",
        1,
    );

    fs::write(&inventory, text).unwrap();

    let error = load_and_validate(&root).unwrap_err().to_string();

    assert!(
        error.contains("defines allowed_vlans but is not VLAN-aware"),
        "{error}"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejects_invalid_vlan_range() {
    let root = temporary_fixture("invalid-vlan");
    let inventory = root.join("inventory.toml");

    let text = fs::read_to_string(&inventory).unwrap().replacen(
        r#"allowed_vlans = "2-4094""#,
        r#"allowed_vlans = "4095""#,
        1,
    );

    fs::write(&inventory, text).unwrap();

    let error = load_and_validate(&root).unwrap_err().to_string();

    assert!(error.contains("outside 1-4094"), "{error}");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejects_missing_management_bridge() {
    let root = temporary_fixture("missing-management");
    let inventory = root.join("inventory.toml");

    let text = fs::read_to_string(&inventory).unwrap().replace(
        r#"location_interface = "lan""#,
        r#"location_interface = "storage""#,
    );

    fs::write(&inventory, text).unwrap();

    let error = load_and_validate(&root).unwrap_err().to_string();

    assert!(
        error.contains("missing location interface 'storage'")
            || error.contains("no bridge attached to management interface"),
        "{error}"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejects_host_address_equal_to_gateway() {
    let root = temporary_fixture("gateway-conflict");
    let location = root.join("locations").join("shane-xfinity.toml");

    let text = fs::read_to_string(&location)
        .unwrap()
        .replace(r#"ipv4 = "10.214.90.30""#, r#"ipv4 = "10.214.90.1""#);

    fs::write(&location, text).unwrap();

    let error = load_and_validate(&root).unwrap_err().to_string();

    assert!(error.contains("conflicts with segment"), "{error}");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn hosts_without_network_profiles_remain_valid() {
    let root = temporary_fixture("optional-profile");
    let inventory = root.join("inventory.toml");

    let text = fs::read_to_string(&inventory).unwrap();

    assert!(text.contains("[hosts.quasar]"));
    load_and_validate(&root).unwrap();

    fs::remove_dir_all(root).unwrap();
}
