use std::path::PathBuf;

use netweft::config::load::ConfigLoader;
use netweft::model::UpstreamScheme;
use netweft::plan::proxy::{ListenerProtocol, resolve_proxy_plan};

fn fixture_config() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/proxy-services/config")
}

#[test]
fn resolves_container_and_cross_host_proxy_intent() {
    let bundle = ConfigLoader::new(&fixture_config()).load(None).unwrap();
    bundle.validate().unwrap();

    let plan = resolve_proxy_plan(&bundle).unwrap();
    assert_eq!(plan.proxies.len(), 2);

    let jellyfin = plan
        .proxies
        .iter()
        .find(|proxy| proxy.id.as_str() == "jellyfin")
        .unwrap();
    assert_eq!(jellyfin.target_host.as_str(), "nexus");
    assert_eq!(jellyfin.upstream.address.to_string(), "10.74.1.10");
    assert_eq!(jellyfin.upstream.port, 8096);
    assert_eq!(jellyfin.upstream.scheme, UpstreamScheme::Http);
    assert_eq!(jellyfin.listeners[0].protocol, ListenerProtocol::Https);
    assert!(jellyfin.websocket);
    assert_eq!(
        jellyfin
            .tls
            .as_ref()
            .unwrap()
            .certificate
            .as_ref()
            .unwrap()
            .id
            .as_str(),
        "wildcard-suhail"
    );

    let dsm = plan
        .proxies
        .iter()
        .find(|proxy| proxy.id.as_str() == "dsm")
        .unwrap();
    assert_eq!(dsm.target_host.as_str(), "nexus");
    assert_eq!(dsm.upstream.address.to_string(), "10.214.90.20");
    assert_eq!(dsm.upstream.port, 5001);
    assert_eq!(dsm.upstream.scheme, UpstreamScheme::Https);
}

#[test]
fn legacy_tls_intent_remains_valid_without_certificate_reference() {
    let config =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/shane-xfinity/config");
    let bundle = ConfigLoader::new(&config).load(None).unwrap();

    bundle.validate().unwrap();
    let plan = resolve_proxy_plan(&bundle).unwrap();
    let jellyfin = plan
        .proxies
        .iter()
        .find(|proxy| proxy.id.as_str() == "jellyfin")
        .unwrap();
    assert!(jellyfin.tls.as_ref().unwrap().certificate.is_none());
}
