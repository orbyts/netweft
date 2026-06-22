use std::collections::BTreeMap;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr};

use ipnet::{Ipv4Net, Ipv6Net};
use serde::Deserialize;

use crate::validate::{ValidationReport, validate_bundle};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettingsFile {
    pub schema_version: u32,
    pub active_location: String,
    #[serde(default)]
    pub paths: PathSettings,
    #[serde(default)]
    pub render: RenderSettings,
    #[serde(default)]
    pub validation: ValidationSettings,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathSettings {
    pub generated_root: Option<String>,
    pub state_root: Option<String>,
    pub cache_root: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RenderSettings {
    pub atomic: bool,
    pub stable_order: bool,
    pub generated_headers: bool,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            atomic: true,
            stable_order: true,
            generated_headers: true,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ValidationSettings {
    pub warn_dynamic_ipv6: bool,
    pub warn_dropbox_runtime: bool,
    pub warn_latest_images: bool,
    pub deny_warnings: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InventoryFile {
    pub schema_version: u32,
    pub domains: Domains,
    pub hosts: BTreeMap<String, Host>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Domains {
    pub primary: String,
    #[serde(default)]
    pub additional: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Host {
    pub kind: HostKind,
    #[serde(default)]
    pub roles: Vec<String>,
    pub parent: Option<String>,
    pub runtime_root: Option<String>,
    pub ssh_user: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostKind {
    Physical,
    Vm,
    Laptop,
    Workstation,
    Nas,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetworksFile {
    pub schema_version: u32,
    pub networks: BTreeMap<String, LogicalNetwork>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogicalNetwork {
    pub kind: NetworkKind,
    pub owner: Option<String>,
    pub docker_name: Option<String>,
    pub ipv4_cidr: Option<Ipv4Net>,
    pub ipv4_gateway: Option<Ipv4Addr>,
    pub allocation_key: String,
    pub preferred_display_id: Option<u16>,
    #[serde(default)]
    pub ula_enabled: bool,
    #[serde(default)]
    pub reverse_dns: bool,
    #[serde(default = "default_true")]
    pub dns_clients: bool,
    pub routing: Option<NetworkRouting>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkRouting {
    pub from: Option<String>,
    pub mode: RoutingMode,
    pub via: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RoutingMode {
    Direct,
    ViaHost,
    HostPrivate,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkKind {
    Lan,
    Docker,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServicesFile {
    pub schema_version: u32,
    pub services: BTreeMap<String, Service>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Service {
    pub kind: ServiceKind,
    pub host: String,
    pub network: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub address: Option<ServiceAddress>,
    #[serde(default)]
    pub ports: Vec<PortMapping>,
    pub runtime: Option<BTreeMap<String, String>>,
    pub ingress: Option<ServiceIngress>,
    pub ssh: Option<SshService>,
    pub web: Option<WebService>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceIngress {
    pub mode: IngressMode,
    pub interface: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum IngressMode {
    HostPort,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceKind {
    Dns,
    ReverseProxy,
    DevelopmentContainer,
    Web,
    Database,
    Generic,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceAddress {
    pub ipv4: Option<Ipv4Addr>,
    pub ipv6_interface_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortMapping {
    pub host: u16,
    pub container: u16,
    pub protocol: TransportProtocol,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransportProtocol {
    Tcp,
    Udp,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SshService {
    pub user: String,
    pub host_port: u16,
    pub container_port: u16,
    pub route: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WebService {
    pub container_port: u16,
    pub domain: String,
    pub access: String,
    pub proxy: String,
    #[serde(default)]
    pub tls: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DnsFile {
    pub schema_version: u32,
    pub dns: DnsSettings,
    #[serde(default)]
    pub zones: Vec<DnsZone>,
    #[serde(default)]
    pub records: Vec<DnsRecord>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DnsSettings {
    pub provider: String,
    pub service: String,
    pub default_ttl: u32,
    pub negative_ttl: u32,
    pub soa: SoaSettings,
    pub recursion: RecursionSettings,
    pub forwarders: DnsForwarders,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SoaSettings {
    pub primary_nameserver: String,
    pub responsible_mailbox: String,
    pub refresh: u32,
    pub retry: u32,
    pub expire: u32,
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RecursionSettings {
    pub enabled: bool,
    pub include_location_segments: bool,
    pub include_tailscale: bool,
    pub include_ula: bool,
    pub include_docker_networks: bool,
}

impl Default for RecursionSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            include_location_segments: true,
            include_tailscale: true,
            include_ula: true,
            include_docker_networks: true,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DnsForwarders {
    #[serde(default)]
    pub ipv4: Vec<Ipv4Addr>,
    #[serde(default)]
    pub ipv6: Vec<std::net::Ipv6Addr>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DnsZone {
    pub name: String,
    pub visibility: DnsVisibility,
    #[serde(default)]
    pub authoritative: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DnsVisibility {
    Internal,
    Public,
    Both,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DnsRecord {
    pub name: String,
    pub kind: DnsRecordKind,
    pub target: Option<String>,
    pub interface: Option<String>,
    pub address_scope: Option<AddressScope>,
    #[serde(default = "default_ipv4_families")]
    pub families: Vec<AddressFamily>,
    #[serde(default)]
    pub reverse: bool,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum DnsRecordKind {
    Host,
    Service,
    Proxy,
    Cname,
    SegmentGateway,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AddressScope {
    Container,
    Ingress,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AddressFamily {
    Ipv4,
    Ipv6,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AllocationsFile {
    pub schema_version: u32,
    pub ula: Option<UlaAllocations>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UlaAllocations {
    pub prefix: Ipv6Net,
    #[serde(default)]
    pub networks: BTreeMap<String, u16>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocationFile {
    pub schema_version: u32,
    pub name: String,
    pub description: Option<String>,
    pub router: Router,
    pub ipv6: LocationIpv6,
    pub segments: BTreeMap<String, Segment>,
    #[serde(default)]
    pub hosts: BTreeMap<String, LocationHost>,
    #[serde(default)]
    pub tailscale: LocationTailscale,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Router {
    pub kind: String,
    pub managed: bool,
    pub supports_vlans: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocationIpv6 {
    pub mode: Ipv6Mode,
    pub prefix: Option<Ipv6Net>,
    pub subnet_prefix_length: Option<u8>,
    pub stability: PrefixStability,
    #[serde(default)]
    pub publish_public_aaaa: bool,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Ipv6Mode {
    Disabled,
    RouterAdvertised,
    Delegated,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrefixStability {
    Dynamic,
    Stable,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Segment {
    pub kind: SegmentKind,
    pub ipv4_cidr: Ipv4Net,
    pub ipv4_gateway: Ipv4Addr,
    pub vlan_id: Option<u16>,
    pub public_ipv6_allocation: Option<u32>,
    #[serde(default = "default_true")]
    pub dns_clients: bool,
    #[serde(default = "default_true")]
    pub reverse_dns: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SegmentKind {
    Lan,
    Vlan,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocationHost {
    #[serde(default)]
    pub interfaces: BTreeMap<String, LocationInterface>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocationInterface {
    pub segment: String,
    pub ipv4: Option<Ipv4Addr>,
    pub ipv6_mode: Option<InterfaceIpv6Mode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InterfaceIpv6Mode {
    Slaac,
    Static,
    Disabled,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocationTailscale {
    #[serde(default)]
    pub enabled: bool,
    pub strategy: Option<TailscaleStrategy>,
    pub primary_router: Option<String>,
    #[serde(default)]
    pub routers: BTreeMap<String, TailscaleRouterSettings>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TailscaleStrategy {
    SubnetRouter,
    HaSubnetRouter,
    DirectNodes,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TailscaleRouterSettings {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub snat_subnet_routes: bool,
    #[serde(default)]
    pub accept_routes: bool,
    #[serde(default)]
    pub exit_node: bool,
    #[serde(default)]
    pub advertise: Vec<String>,
}

#[derive(Debug)]
pub struct ConfigBundle {
    pub settings: SettingsFile,
    pub inventory: InventoryFile,
    pub networks: NetworksFile,
    pub services: ServicesFile,
    pub dns: DnsFile,
    pub allocations: AllocationsFile,
    pub location: LocationFile,
}

impl ConfigBundle {
    pub fn validate(&self) -> anyhow::Result<ValidationReport> {
        validate_bundle(self)
    }

    pub fn print_summary(&self) {
        println!("Location: {}", self.location.name);
        println!("Router: {}", self.location.router.kind);
        println!("IPv6 mode: {}", self.location.ipv6.mode);
        println!("Hosts: {}", self.inventory.hosts.len());
        println!("Networks: {}", self.networks.networks.len());
        println!("Services: {}", self.services.services.len());
        println!("DNS zones: {}", self.dns.zones.len());
    }

    pub fn print_hosts(&self) {
        for (name, host) in &self.inventory.hosts {
            println!(
                "{name}\t{:?}\t{}\t{}",
                host.kind,
                if host.enabled { "enabled" } else { "disabled" },
                host.roles.join(",")
            );
        }
    }

    pub fn print_networks(&self) {
        for (name, network) in &self.networks.networks {
            println!(
                "{name}\t{:?}\t{}",
                network.kind,
                network
                    .ipv4_cidr
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "-".to_owned())
            );
        }
    }

    pub fn print_services(&self) {
        for (name, service) in &self.services.services {
            println!(
                "{name}\t{:?}\thost={}\tnetwork={}",
                service.kind, service.host, service.network
            );
        }
    }
}

impl fmt::Display for Ipv6Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Disabled => "disabled",
            Self::RouterAdvertised => "router-advertised",
            Self::Delegated => "delegated",
        };
        f.write_str(value)
    }
}

pub fn parse_ip(value: &str) -> Result<IpAddr, std::net::AddrParseError> {
    value.parse()
}

fn default_true() -> bool {
    true
}

fn default_ipv4_families() -> Vec<AddressFamily> {
    vec![AddressFamily::Ipv4]
}
