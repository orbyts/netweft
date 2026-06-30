# Netweft guide

Netweft turns a description of your infrastructure into validated, deterministic configuration artifacts. This guide is written for someone installing Netweft for the first time and wanting to understand both **what to configure** and **why the network works that way**.

The running example is a portable homelab with an unmanaged consumer router, two Proxmox nodes, a NAS, a small infrastructure host, client machines, containers, internal DNS, reverse proxying, SSH aliases, Tailscale, and Cloudflare Tunnel ingress.

## 1. The mental model

Netweft separates infrastructure into four layers:

```text
stable identity
    inventory.toml, networks.toml, services.toml

site-specific attachment
    locations/<name>.toml

provider-neutral resolved plans
    host networking, DNS, proxy, guests, storage, mounts, SSH, ingress

provider-specific artifacts
    Proxmox, Netplan, BIND, Nginx, Docker, systemd, Cloudflare, OpenSSH
```

That separation matters because a host such as `zion` is still the same host when it moves between homes, but its address, gateway, and available IPv6 prefix may change.

Netweft is a **planner and renderer**, not a remote orchestration system. It validates references and emits files or guarded scripts. Deployment remains an explicit operator action.

## 2. Example topology

```text
Internet
   │
   ├── Cloudflare edge
   │      └── named Tunnel → nexus → Nginx → private services
   │
Xfinity XB8 router
10.214.90.1/24
   │
   └── main LAN: 10.214.90.0/24
          ├── nexus       10.214.90.10  DNS, proxy, Docker, Tailscale
          ├── ds1621plus  10.214.90.20  NAS
          ├── zion        10.214.90.30  Proxmox
          ├── atlas       10.214.90.40  Proxmox
          ├── summit      10.214.90.50  future Proxmox node
          └── quasar      10.214.90.60  workstation

zion guests
   ├── flicker  10.214.90.31
   ├── whisk    10.214.90.32
   └── vortex   10.214.90.35

logical networks
   ├── nexus-containers   10.78.0.0/16
   ├── vortex-containers  10.74.1.0/24
   └── shasta SDN         10.29.100.0/24
```

### What `/24` means

`10.214.90.0/24` is CIDR notation. The `/24` means the first 24 bits identify the network. In familiar subnet-mask form, that is `255.255.255.0`.

For this network:

- network address: `10.214.90.0`;
- usable host range: normally `10.214.90.1` through `10.214.90.254`;
- broadcast address: `10.214.90.255`;
- gateway: `10.214.90.1`.

A gateway is the router a host uses to reach destinations outside its directly connected subnet.

### Bridges

A Linux bridge behaves like a software Ethernet switch. On Proxmox, `vmbr0` normally joins a physical NIC and the host/guest interfaces into one Layer-2 network. A bridge can be VLAN-aware even when the current router does not provide VLANs; that means the bridge can carry tagged frames if the surrounding network later supports them.

### Stable IPv4 versus dynamic IPv6

The example location uses stable private IPv4 addresses but receives only a router-advertised dynamic IPv6 `/64`. Netweft may allow that prefix for reachability and recursion ACLs, but it must not treat it as durable allocatable infrastructure identity. Stable IPv6 allocation belongs in ULA space or in a delegated prefix that the operator controls.

## 3. Install Netweft

```bash
cargo install netweft --version 0.2.0 --locked
netweft --version
```

The default configuration directory is:

```text
~/.config/netweft
```

Override it with:

```bash
export NETWEFT_CONFIG_DIR=/path/to/netweft
```

Inspect all resolved paths:

```bash
netweft paths
```

## 4. Create the configuration tree

```bash
mkdir -p ~/.config/netweft/locations
```

Core files:

```text
~/.config/netweft/
├── netweft.toml
├── inventory.toml
├── networks.toml
├── services.toml
├── dns.toml
├── allocations.toml
└── locations/
    └── shane-xfinity.toml
```

Optional subsystems add files such as:

```text
cloudflare.toml
docker.toml
guests.toml
mounts.toml
nas-permissions.toml
proxmox-sdn.toml
proxmox-storages.toml
ssh.toml
```

Every configuration struct rejects unknown keys. A typo is therefore an error instead of being silently ignored.

## 5. Configure Netweft itself

```toml
# netweft.toml
schema_version = 1
active_location = "shane-xfinity"

[paths]
generated_root = "~/.local/share/netweft/generated"
state_root = "~/.local/state/netweft"
cache_root = "~/.cache/netweft"

[render]
atomic = true
stable_order = true
generated_headers = true

[validation]
warn_dynamic_ipv6 = true
warn_dropbox_runtime = true
warn_latest_images = true
deny_warnings = false
```

`active_location` selects which site-specific file participates in resolution. The path settings control where generated artifacts and state are stored. Atomic rendering prevents partially written output. Stable ordering keeps diffs reproducible.

Full key reference: [netweft.toml](configuration/NETWEFT.md).

## 6. Define stable host identity

```toml
# inventory.toml
schema_version = 1

[domains]
primary = "suhail.ink"
additional = ["suhail.photos", "suhail.art", "suhail.life"]

[hosts.nexus]
kind = "physical"
roles = ["infrastructure", "dns", "proxy", "docker", "tailscale"]
runtime_root = "/var/lib/suhail/services/nexus"
ssh_user = "suhail"
enabled = true

[hosts.zion]
kind = "physical"
roles = ["proxmox"]
ssh_user = "root"
enabled = true
```

A host name such as `zion` is a stable identifier. Do not put the current site's IP address in this basic host table. The address belongs in the location file.

### Proxmox host networking

A Proxmox host also declares its physical links and bridges:

```toml
[hosts.atlas.network]
provider = "proxmox-ifupdown2"
management_interface = "lan"
preserve_includes = true

[[hosts.atlas.network.links]]
name = "enp2s0f0np0"
kind = "ethernet"

[[hosts.atlas.network.bridges]]
name = "vmbr0"
ports = ["enp2s0f0np0"]
location_interface = "lan"
vlan_aware = true
allowed_vlans = "2-4094"
stp = false
forward_delay = 2
comment = "Primary"
```

`location_interface = "lan"` connects the provider-specific bridge to the site-specific interface named `lan`. The location file supplies the actual address and gateway.

`preserve_includes = true` keeps the standard `source /etc/network/interfaces.d/*` line in generated ifupdown2 configuration.

Full key reference: [inventory.toml](configuration/INVENTORY.md).

## 7. Define the location

```toml
# locations/shane-xfinity.toml
schema_version = 1
name = "shane-xfinity"
description = "Xfinity XB8 network in San Francisco"

[router]
kind = "xfinity-xb8"
managed = false
supports_vlans = false

[ipv6]
mode = "router-advertised"
prefix = "2601:645:8a02:6610::/64"
stability = "dynamic"
publish_public_aaaa = false

[segments.main]
kind = "lan"
ipv4_cidr = "10.214.90.0/24"
ipv4_gateway = "10.214.90.1"
vlan_id = 0

[hosts.nexus.interfaces.lan]
segment = "main"
ipv4 = "10.214.90.10"
ipv6_mode = "slaac"
ula_interface_id = "10"

[hosts.zion.interfaces.lan]
segment = "main"
ipv4 = "10.214.90.30"
ipv6_mode = "slaac"
ula_interface_id = "30"

[hosts.atlas.interfaces.lan]
segment = "main"
ipv4 = "10.214.90.40"
ipv6_mode = "slaac"
ula_interface_id = "40"
```

A segment describes one connected IP network. Each host interface points to a segment and receives its site-specific address there.

`ipv6_mode = "slaac"` means the operating system obtains a global IPv6 address from router advertisements. `ula_interface_id` is a stable identifier used when Netweft derives a ULA address from `allocations.toml`.

Full key reference: [location files](configuration/LOCATIONS.md).

## 8. Define logical networks

```toml
# networks.toml
schema_version = 1

[networks.nexus-containers]
kind = "docker"
owner = "nexus"
docker_name = "fuji"
ipv4_cidr = "10.78.0.0/16"
ipv4_gateway = "10.78.0.1"
allocation_key = "nexus-containers"
preferred_display_id = 121
ula_enabled = true
reverse_dns = true

[networks.nexus-containers.routing]
from = "nexus"
mode = "direct"
```

A logical network is not necessarily the same thing as a physical LAN. It may be a Docker bridge or another provider-specific network. `owner` identifies the host that owns it. Routing metadata tells Netweft how another host can reach it.

Full key reference: [networks.toml](configuration/NETWORKS.md).

## 9. Define services and proxy intent

```toml
# services.toml
[services.nginx]
kind = "reverse-proxy"
host = "nexus"
network = "nexus-containers"
enabled = true

[services.nginx.address]
ipv4 = "10.78.88.88"
ipv6_interface_id = "8888"

[services.nginx.ingress]
mode = "host-port"
interface = "lan"

[[services.nginx.ports]]
host = 443
container = 443
protocol = "tcp"
```

A service has stable placement (`host`), network membership (`network`), optional logical addresses, and optional host-port ingress.

A web service describes provider-neutral reverse-proxy intent:

```toml
[services.zion-ui]
kind = "web"
host = "zion"
network = "nexus-containers"
enabled = true

[services.zion-ui.web]
container_port = 8006
domain = "zion.suhail.ink"
access = "reverse-proxy"
proxy = "nginx"
scheme = "https"
tls = true
certificate = "wildcard-suhail"
force_https = true
websocket = true
```

This does not mean Zion is literally attached to the Docker network. The proxy resolver uses host identity and location attachment to derive the reachable upstream, while the Nginx adapter turns that plan into server blocks.

Netweft stores certificate **references**, not certificate material:

```toml
[certificates.wildcard-suhail]
domains = ["suhail.ink", "*.suhail.ink"]
certificate_path = "/etc/netweft/certificates/wildcard-suhail/fullchain.pem"
private_key_path = "/etc/netweft/certificates/wildcard-suhail/privkey.pem"
```

Full key reference: [services.toml](configuration/SERVICES.md).

## 10. Configure DNS

```toml
# dns.toml
schema_version = 1

[dns]
provider = "bind9"
service = "bind9"
default_ttl = 86400
negative_ttl = 604800

[dns.recursion]
enabled = true
include_location_segments = true
include_tailscale = true
include_ula = true
include_docker_networks = true

[dns.forwarders]
ipv4 = ["1.1.1.1", "1.0.0.1"]
ipv6 = ["2606:4700:4700::1111", "2606:4700:4700::1001"]

[[records]]
name = "zion.suhail.ink"
kind = "host"
target = "zion"
interface = "lan"
families = ["ipv4"]
reverse = true
```

An authoritative DNS server answers for zones it owns. A recursive resolver looks up names on behalf of clients. Forwarders are upstream recursive resolvers. Reverse DNS maps an address back to a name.

The recursion include flags derive ACLs from the resolved topology. Merely declaring a network is not enough; Netweft also considers whether it is reachable from the DNS host.

Full key reference: [dns.toml](configuration/DNS.md).

## 11. Configure durable ULA allocation

```toml
# allocations.toml
schema_version = 1

[ula]
prefix = "fdd7:d134:61e3::/48"

[ula.segments]
main = 256

[ula.networks]
nexus-containers = 289
vortex-containers = 321
```

ULA addresses are private IPv6 addresses, roughly analogous to RFC1918 private IPv4 space. The allocation numbers are durable identity. Do not renumber them casually after deployment.

Full key reference: [allocations.toml](configuration/ALLOCATIONS.md).

## 12. Add optional subsystems

### Guests

```toml
# guests.toml
[guests.vortex]
kind = "vm"
host = "zion"
vmid = 212
mac = "BC:24:11:4C:28:D9"
interface = "lan"
bridge = "vmbr0"
ipv4 = "10.214.90.35"
address_mode = "reserved-dhcp"
onboot = true
startup = "order=10,up=30,down=60"
```

`reserved-dhcp` means Netweft models a stable reservation rather than writing a static address inside the guest.

Reference: [guests.toml](configuration/GUESTS.md).

### Proxmox SDN

```toml
[zones.pacific]
host = "zion"
kind = "simple"
ipam = "pve"
dhcp = "dnsmasq"

[vnets.shasta]
zone = "pacific"
alias = "shasta"

[[vnets.shasta.subnets]]
cidr = "10.29.100.0/24"
gateway = "10.29.100.1"
dhcp_start = "10.29.100.100"
dhcp_end = "10.29.100.254"
snat = true
```

Reference: [proxmox-sdn.toml](configuration/PROXMOX-SDN.md).

### Proxmox storage

```toml
[storages.synology]
host = "zion"
storage_id = "Synology"
provider = "nfs"
server_host = "ds1621plus"
export = "/volume1/iso"
mount_path = "/mnt/pve/Synology"
options = ["vers=4.1", "proto=tcp"]
content = ["images", "backup", "iso", "vztmpl"]
prune_backups = "keep-all=1"
```

Reference: [proxmox-storages.toml](configuration/PROXMOX-STORAGES.md).

### Network mounts

```toml
[mounts.vortex-datalib]
host = "vortex"
provider = "nfs"
server_host = "ds1621plus"
export = "/volume1/dataLib"
mount_path = "/mnt/dataLib"
options = ["vers=4.1", "proto=tcp", "rw", "_netdev", "nofail"]
required_by = ["docker.service"]
```

Reference: [mounts.toml](configuration/MOUNTS.md).

### SSH clients

```toml
[clients.quasar.identities.proxmox]
file = "~/.ssh/id_proxmox"

[targets.zion]
host = "zion"
interface = "lan"
user = "root"
port = 22
identity = "proxmox"
forward_agent = false
```

Reference: [ssh.toml](configuration/SSH.md).

### Cloudflare ingress

```toml
[providers.suhail-ink]
zone = "suhail.ink"
zone_id = "..."
account_id = "..."
tunnel_api_token_env = "CLOUDFLARE_API_TOKEN_SUHAIL_INK_TUNNEL"
dns_api_token_env = "CLOUDFLARE_API_TOKEN_SUHAIL_INK_DNS"

[tunnels.nexus-ingress]
provider = "suhail-ink"
connector_host = "nexus"
origin = "https://127.0.0.1:443"
origin_tls_verify = false
hostnames = ["dsm.suhail.ink", "zion.suhail.ink"]
```

Only environment-variable names belong in this file. Token values remain in a secret manager or local secret environment file.

Reference: [cloudflare.toml](configuration/CLOUDFLARE.md).

## 13. Validate before rendering

```bash
netweft validate
netweft adapters list
```

Inspect derived plans before generating files:

```bash
netweft show dns
netweft show proxy
netweft show host-network --host atlas
netweft show guests
netweft show proxmox-sdn --host zion
netweft show proxmox-storage --host zion
netweft show ssh --client quasar
netweft show cloudflare
```

The `show` commands answer “what did Netweft resolve?” The `render` commands answer “what files or scripts would implement it?”

## 14. Render and understand the output contract

Different adapters intentionally emit different artifact types. Do not assume every output directory contains `install.sh`.

| Adapter | Main output | `apply`/`install` | `verify` | `rollback` |
|---|---|---:|---:|---:|
| `bind` | BIND configuration and zone files | no | no | no |
| `nginx` | `nginx.conf` and server blocks | no | no | no |
| `env` | shell and Compose environment files | no | no | no |
| `proxmox` | `/etc/network/interfaces`, `/etc/hosts`, `/etc/resolv.conf` payload | no | no | no |
| `synology-nfs-permissions` | manual action plan | no | no | no |
| `docker` | desired daemon/network state | yes | yes | yes |
| `netplan` | Netplan and cloud-init payload | yes | no dedicated script | yes |
| `proxmox-guests` | reconciliation plan | yes | adapter-specific | yes |
| `proxmox-sdn` | SDN reconciliation plan | yes | yes | yes |
| `proxmox-storage` | storage fragment and reconciliation | yes | adapter-specific | yes |
| `systemd-mounts` | units and service drop-ins | yes | adapter-specific | adapter-specific |
| `ssh` | OpenSSH include fragments | install | yes | yes |
| `cloudflare` | tunnel/DNS plan and connector bundle | install/apply | yes | yes |

Always inspect the actual rendered tree:

```bash
find ~/.local/share/netweft/generated -type f | sort
```

### Example: Proxmox host networking

```bash
netweft render proxmox --host atlas
```

Output:

```text
.../hosts/atlas/proxmox/
├── etc/
│   ├── hosts
│   ├── network/interfaces
│   └── resolv.conf
└── manifest.txt
```

This adapter emits a filesystem payload only. Deploy it by backing up the target files, staging the generated `etc/` tree, validating it, copying the files into place, and rebooting or activating networking under controlled conditions. See [Proxmox deployment](deployment/PROXMOX.md).

### Example: Cloudflare ingress

```bash
netweft render cloudflare --tunnel nexus-ingress
```

This adapter emits an action plan, API reconciliation script, connector Compose file, install script, verifier, and rollback script. The generated runtime credential file is secret and must not be committed.

## 15. Safe deployment workflow

```text
validate
→ inspect resolved plan
→ render
→ inspect generated tree
→ back up target state
→ stage artifacts on target
→ run provider-native validation
→ apply explicitly
→ verify service and network behavior
→ retain rollback path
```

For remote network changes, preserve an out-of-band console whenever possible. Proxmox networking changes can disconnect SSH. Cluster communication such as Corosync must be migrated in an order that keeps both old and new paths available until quorum is restored.

Deployment guides: [deployment index](deployment/README.md).

## 16. Common mistakes

### Putting site addresses in inventory

Incorrect: embedding the current home address directly in `[hosts.zion]`.

Correct: define `zion` in `inventory.toml`, then attach `zion.interfaces.lan` in the active location.

### Treating a dynamic ISP prefix as durable identity

A router-advertised `/64` may change. Do not publish long-lived AAAA records or use it as a stable Corosync identity unless the prefix is under your control.

### Assuming every adapter deploys itself

Rendering Proxmox host networking produces files, not scripts. Rendering Cloudflare produces guarded scripts. Always inspect the generated tree and consult the matching deployment guide.

### Committing secrets

Configuration files may contain environment-variable names, zone IDs, and account IDs. They must not contain API token values, private keys, connector credentials, or generated `deployment.env` files.

### Applying before inspecting the resolved plan

Use `show` first. A valid TOML file can still express the wrong operational intent.

## 17. Where to go next

- [Configuration reference](configuration/README.md): every supported TOML file and key.
- [Adapters](adapters/README.md): what each renderer consumes and emits.
- [Deployment](deployment/README.md): provider-specific activation procedures.
- [Operations](operations/README.md): verification, troubleshooting, and rollback.
- [Architecture](ARCHITECTURE.md): internal planning and adapter boundaries.
