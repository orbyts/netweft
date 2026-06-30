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
    #[serde(default)]
    pub network: Option<HostNetworkConfig>,
    #[serde(default)]
    pub os_network: Option<HostOsNetworkConfig>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Stable operating-system network topology attached to a host.
///
/// Addresses and gateways remain location-specific. This structure describes
/// host properties that normally travel with the machine: physical links,
/// bridges, and the operating-system network provider.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostNetworkConfig {
    pub provider: HostNetworkProvider,
    pub management_interface: String,
    #[serde(default)]
    pub preserve_includes: bool,
    #[serde(default)]
    pub links: Vec<HostNetworkLink>,
    #[serde(default)]
    pub bridges: Vec<HostNetworkBridge>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum HostNetworkProvider {
    ProxmoxIfupdown2,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostOsNetworkConfig {
    pub provider: HostOsNetworkProvider,
    pub renderer: HostOsNetworkRenderer,
    pub interface: String,
    pub ipv4_mode: HostOsIpv4Mode,
    pub ipv6_mode: HostOsIpv6Mode,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum HostOsNetworkProvider {
    Netplan,
}

impl HostOsNetworkProvider {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Netplan => "netplan",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum HostOsNetworkRenderer {
    Networkd,
}

impl HostOsNetworkRenderer {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Networkd => "networkd",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum HostOsIpv4Mode {
    Dhcp,
}

impl HostOsIpv4Mode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dhcp => "dhcp",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum HostOsIpv6Mode {
    Disabled,
    RouterAdvertised,
}

impl HostOsIpv6Mode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::RouterAdvertised => "router-advertised",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostNetworkLink {
    pub name: String,
    pub kind: HostNetworkLinkKind,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum HostNetworkLinkKind {
    Ethernet,
    Wifi,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostNetworkBridge {
    pub name: String,
    #[serde(default)]
    pub ports: Vec<String>,
    pub location_interface: Option<String>,
    #[serde(default)]
    pub vlan_aware: bool,
    pub allowed_vlans: Option<String>,
    #[serde(default)]
    pub stp: bool,
    #[serde(default)]
    pub forward_delay: u32,
    pub comment: Option<String>,
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
pub struct SshFile {
    pub schema_version: u32,
    #[serde(default)]
    pub clients: BTreeMap<String, SshClient>,
    #[serde(default)]
    pub targets: BTreeMap<String, SshTarget>,
}

impl Default for SshFile {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            clients: BTreeMap::new(),
            targets: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SshClient {
    #[serde(default)]
    pub identities: BTreeMap<String, SshIdentity>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SshIdentity {
    pub file: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SshTarget {
    pub host: Option<String>,
    pub guest: Option<String>,
    pub service: Option<String>,
    pub interface: Option<String>,
    pub user: String,
    #[serde(default = "default_ssh_port")]
    pub port: u16,
    pub identity: String,
    #[serde(default)]
    pub forward_agent: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CloudflareFile {
    pub schema_version: u32,
    #[serde(default)]
    pub providers: BTreeMap<String, CloudflareProvider>,
    #[serde(default)]
    pub tunnels: BTreeMap<String, CloudflareTunnel>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CloudflareProvider {
    pub zone: String,
    pub zone_id: String,
    pub account_id: String,
    pub tunnel_api_token_env: String,
    pub dns_api_token_env: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CloudflareTunnel {
    pub provider: String,
    pub connector_host: String,
    pub origin: String,
    #[serde(default = "default_true")]
    pub origin_tls_verify: bool,
    #[serde(default)]
    pub hostnames: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocationExternalIngress {
    pub mode: ExternalIngressMode,
    pub provider: String,
    pub tunnel: Option<String>,
    pub origin_host: Option<String>,
    #[serde(default)]
    pub publish_ipv4: bool,
    #[serde(default)]
    pub publish_ipv6: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ExternalIngressMode {
    CloudflareTunnel,
    CloudflareDirect,
    Disabled,
}

impl ExternalIngressMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CloudflareTunnel => "cloudflare-tunnel",
            Self::CloudflareDirect => "cloudflare-direct",
            Self::Disabled => "disabled",
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DockerFile {
    pub schema_version: u32,
    #[serde(default)]
    pub hosts: BTreeMap<String, DockerHost>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DockerHost {
    pub bridge_ipv4_cidr: Ipv4Net,
    pub bridge_ipv6_cidr: Ipv6Net,
    pub ipv4_pool_base: Ipv4Net,
    pub ipv4_pool_size: u8,
    pub ipv6_pool_base: Ipv6Net,
    pub ipv6_pool_size: u8,
    #[serde(default)]
    pub network_migrations: BTreeMap<String, DockerNetworkMigration>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DockerNetworkMigration {
    pub previous_ipv6_cidr: Option<Ipv6Net>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProxmoxSdnFile {
    pub schema_version: u32,
    #[serde(default)]
    pub zones: BTreeMap<String, ProxmoxSdnZone>,
    #[serde(default)]
    pub vnets: BTreeMap<String, ProxmoxSdnVnet>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProxmoxSdnZone {
    pub host: String,
    pub kind: ProxmoxSdnZoneKind,
    pub ipam: ProxmoxSdnIpam,
    pub dhcp: ProxmoxSdnDhcp,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProxmoxSdnZoneKind {
    Simple,
}

impl ProxmoxSdnZoneKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Simple => "simple",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProxmoxSdnIpam {
    Pve,
}

impl ProxmoxSdnIpam {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pve => "pve",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProxmoxSdnDhcp {
    Dnsmasq,
}

impl ProxmoxSdnDhcp {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dnsmasq => "dnsmasq",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProxmoxSdnVnet {
    pub zone: String,
    pub alias: Option<String>,
    #[serde(default)]
    pub subnets: Vec<ProxmoxSdnSubnet>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProxmoxSdnSubnet {
    pub cidr: Ipv4Net,
    pub gateway: Ipv4Addr,
    pub dhcp_start: Ipv4Addr,
    pub dhcp_end: Ipv4Addr,
    #[serde(default)]
    pub snat: bool,
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

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkKind {
    Lan,
    Docker,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServicesFile {
    pub schema_version: u32,
    #[serde(default)]
    pub certificates: BTreeMap<String, CertificateReference>,
    pub services: BTreeMap<String, Service>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CertificateReference {
    #[serde(default)]
    pub domains: Vec<String>,
    pub certificate_path: String,
    pub private_key_path: String,
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
    pub scheme: UpstreamScheme,
    #[serde(default)]
    pub tls: bool,
    pub certificate: Option<String>,
    #[serde(default)]
    pub force_https: bool,
    #[serde(default)]
    pub websocket: bool,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UpstreamScheme {
    #[default]
    Http,
    Https,
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
pub struct GuestsFile {
    pub schema_version: u32,
    #[serde(default)]
    pub guests: BTreeMap<String, Guest>,
}

impl Default for GuestsFile {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            guests: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Guest {
    pub kind: GuestKind,
    pub host: String,
    pub vmid: u32,
    pub mac: String,
    pub interface: String,
    #[serde(default = "default_guest_bridge")]
    pub bridge: String,
    pub ipv4: Ipv4Addr,
    pub address_mode: GuestAddressMode,
    #[serde(default)]
    pub onboot: bool,
    pub startup: Option<String>,
    #[serde(default)]
    pub firewall: bool,
    #[serde(default)]
    pub pci_devices: Vec<GuestPciDevice>,
    #[serde(default)]
    pub virtiofs: Vec<GuestVirtioFs>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestPciDevice {
    pub slot: u8,
    pub device: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestVirtioFs {
    pub slot: u8,
    pub directory: String,
    #[serde(default)]
    pub expose_acl: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GuestKind {
    Vm,
    Lxc,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum GuestAddressMode {
    ReservedDhcp,
    Static,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkMountsFile {
    pub schema_version: u32,
    #[serde(default)]
    pub mounts: BTreeMap<String, NetworkMount>,
}

impl Default for NetworkMountsFile {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            mounts: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkMount {
    pub host: String,
    pub provider: NetworkMountProvider,
    pub server_host: String,
    pub export: String,
    pub mount_path: String,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub required_by: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NetworkMountProvider {
    Nfs,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NasPermissionsFile {
    pub schema_version: u32,
    #[serde(default)]
    pub permissions: BTreeMap<String, NasPermission>,
}

impl Default for NasPermissionsFile {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            permissions: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NasPermission {
    pub nas: String,
    pub provider: NasPermissionProvider,
    pub share: String,
    pub client_host: String,
    pub access: NasAccess,
    pub squash: NasSquash,
    #[serde(default = "default_nfs_security")]
    pub security: String,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum NasPermissionProvider {
    SynologyNfs,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum NasAccess {
    ReadOnly,
    ReadWrite,
}

impl NasAccess {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read-only",
            Self::ReadWrite => "read-write",
        }
    }
    pub fn dsm_name(self) -> &'static str {
        match self {
            Self::ReadOnly => "Read-Only",
            Self::ReadWrite => "Read/Write",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum NasSquash {
    None,
    MapRootToAdmin,
    MapAllToAdmin,
    MapAllToGuest,
}

impl NasSquash {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::MapRootToAdmin => "map-root-to-admin",
            Self::MapAllToAdmin => "map-all-to-admin",
            Self::MapAllToGuest => "map-all-to-guest",
        }
    }
    pub fn dsm_name(self) -> &'static str {
        match self {
            Self::None => "No mapping",
            Self::MapRootToAdmin => "Map root to admin",
            Self::MapAllToAdmin => "Map all users to admin",
            Self::MapAllToGuest => "Map all users to guest",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProxmoxStoragesFile {
    pub schema_version: u32,
    #[serde(default)]
    pub storages: BTreeMap<String, ProxmoxStorage>,
}

impl Default for ProxmoxStoragesFile {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            storages: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProxmoxStorage {
    pub host: String,
    pub storage_id: String,
    pub provider: ProxmoxStorageProvider,
    pub server_host: String,
    pub export: String,
    pub mount_path: String,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub content: Vec<String>,
    pub prune_backups: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProxmoxStorageProvider {
    Nfs,
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
    pub segments: BTreeMap<String, u16>,
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
    pub external_ingress: Option<LocationExternalIngress>,
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
    pub ula_interface_id: Option<String>,
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
    pub docker: DockerFile,
    pub ssh: SshFile,
    pub cloudflare: CloudflareFile,
    pub services: ServicesFile,
    pub guests: GuestsFile,
    pub mounts: NetworkMountsFile,
    pub nas_permissions: NasPermissionsFile,
    pub proxmox_storages: ProxmoxStoragesFile,
    pub proxmox_sdn: ProxmoxSdnFile,
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

fn default_ssh_port() -> u16 {
    22
}

fn default_true() -> bool {
    true
}

fn default_guest_bridge() -> String {
    "vmbr0".to_owned()
}

fn default_nfs_security() -> String {
    "sys".to_owned()
}

fn default_ipv4_families() -> Vec<AddressFamily> {
    vec![AddressFamily::Ipv4]
}

pub mod id;
