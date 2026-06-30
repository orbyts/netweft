# Build a portable homelab: complete example

This chapter starts with an empty Netweft configuration and builds a real multi-host topology. It is intentionally opinionated: copy the structure, then replace names and addresses with your own.

## The topology

```text
Internet
   │
Xfinity XB8 (unmanaged router, 10.214.90.1)
   │
   └── main LAN: 10.214.90.0/24
       ├── nexus       10.214.90.10  DNS, proxy, Docker, Tailscale
       ├── ds1621plus  10.214.90.20  NAS
       ├── zion        10.214.90.30  Proxmox
       ├── atlas       10.214.90.40  Proxmox
       ├── summit      10.214.90.50  future Proxmox node
       └── quasar      10.214.90.60  workstation
```

A `/24` contains 256 IPv4 addresses. In this example, `10.214.90.0` identifies the network, `10.214.90.255` is the broadcast address, and usable host addresses are normally `.1` through `.254`. The router is `.1`; stable infrastructure addresses are grouped in increments of ten.

Netweft separates **identity** from **placement**:

- `inventory.toml` says that `atlas` exists and is a Proxmox host;
- `locations/shane-xfinity.toml` says that, at this site, Atlas uses `10.214.90.40`;
- adapter-specific files describe optional Docker, Proxmox, SSH, DNS, and Cloudflare behavior.

That separation lets the same host move to another location without rewriting its stable identity.

## 1. Select the active location

Create `~/.config/netweft/netweft.toml`:

```toml
schema_version = 1
active_location = "shane-xfinity"

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

`active_location` selects `locations/shane-xfinity.toml`. A command-line `--location` override changes one invocation only.

## 2. Define stable hosts

Create `inventory.toml`:

```toml
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

[hosts.ds1621plus]
kind = "nas"
roles = ["storage"]
enabled = true

[hosts.zion]
kind = "physical"
roles = ["proxmox"]
ssh_user = "root"
enabled = true

[hosts.atlas]
kind = "physical"
roles = ["proxmox"]
ssh_user = "root"
enabled = true

[hosts.summit]
kind = "physical"
roles = ["proxmox"]
ssh_user = "root"
enabled = false

[hosts.quasar]
kind = "workstation"
roles = ["client", "development"]
ssh_user = "suhail"
enabled = true
```

`enabled = false` preserves a planned identity without including it in active plans.

## 3. Describe Proxmox host networking

Host-network topology belongs under the inventory host because NIC and bridge names travel with the machine.

```toml
[hosts.atlas.network]
provider = "proxmox-ifupdown2"
management_interface = "lan"
preserve_includes = true

[[hosts.atlas.network.links]]
name = "enp2s0f0np0"
kind = "ethernet"

[[hosts.atlas.network.links]]
name = "enp2s0f1np1"
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

[[hosts.atlas.network.bridges]]
name = "vmbr1"
ports = []
vlan_aware = true
allowed_vlans = "2-4094"
stp = false
forward_delay = 2
comment = "VM & CT Network"
```

A Linux bridge behaves like a software Ethernet switch. `vmbr0` joins the physical uplink to the host and guests. `location_interface = "lan"` tells Netweft to place the address resolved from the location's `lan` attachment on this bridge.

`vlan_aware = true` allows tagged VLAN traffic through the bridge. It does not create VLANs by itself. `allowed_vlans` limits tags accepted by the provider.

The Proxmox adapter renders a filesystem payload only:

```text
hosts/atlas/proxmox/
├── etc/hosts
├── etc/network/interfaces
├── etc/resolv.conf
└── manifest.txt
```

It does not emit an installer. Inspect, back up, stage, and install those files explicitly.

## 4. Define the physical location

Create `locations/shane-xfinity.toml`:

```toml
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

[hosts.ds1621plus.interfaces.lan]
segment = "main"
ipv4 = "10.214.90.20"
ipv6_mode = "slaac"
ula_interface_id = "20"

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

[hosts.quasar.interfaces.lan]
segment = "main"
ipv4 = "10.214.90.60"
ipv6_mode = "slaac"
ula_interface_id = "60"
```

### Why IPv6 is marked dynamic

Router-advertised IPv6 means the router announces a prefix and hosts form addresses using SLAAC. At this site the ISP can change that `/64`; Netweft may use it for current reachability and DNS recursion ACLs, but it must not treat it as durable allocation space. Therefore `publish_public_aaaa = false`.

A delegated prefix is different: with DHCPv6 prefix delegation, the router gives you a larger prefix such as `/56`, which can be divided into stable `/64` networks.

## 5. Define logical networks

Create `networks.toml`:

```toml
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

A logical Docker network is not the same as the physical LAN. `routing.mode = "direct"` means the named `from` host can reach it directly. Use `via-host` when another machine is the next hop, or `host-private` when the network must not be advertised externally.

## 6. Add durable ULA allocations

Create `allocations.toml`:

```toml
schema_version = 1

[ula]
global_id = "example-global-id"

[ula.network_ids]
nexus-containers = 121
```

ULA space is private IPv6, analogous in purpose to RFC1918 private IPv4. The allocation file preserves stable identifiers so regenerated plans do not renumber networks.

## 7. Place services and declare proxy intent

Create `services.toml`:

```toml
schema_version = 1

[certificates.wildcard-suhail]
domains = ["suhail.ink", "*.suhail.ink"]
certificate_path = "/etc/netweft/certificates/wildcard-suhail/fullchain.pem"
private_key_path = "/etc/netweft/certificates/wildcard-suhail/privkey.pem"

[services.nginx]
kind = "reverse-proxy"
host = "nexus"
network = "nexus-containers"
enabled = true

[services.dsm]
kind = "web"
host = "ds1621plus"
network = "nexus-containers"
enabled = true

[services.dsm.web]
domain = "dsm.suhail.ink"
access = "reverse-proxy"
proxy = "nginx"
scheme = "https"
upstream_host = "10.214.90.20"
upstream_port = 5001
tls = true
certificate = "wildcard-suhail"
force_https = true
websocket = true
```

A service describes intent. The proxy adapter turns that intent into provider-specific configuration. Certificate entries are references only; Netweft does not issue or renew certificates.

## 8. Configure DNS

Create `dns.toml` with authoritative zones, recursion policy, and records. Prefer references to hosts and services over copying addresses where the model supports them.

```toml
schema_version = 1

[settings]
provider = "bind"
server_host = "nexus"

[settings.recursion]
enabled = true
allow = ["segment:main", "network:nexus-containers"]

[zones.suhail-ink]
name = "suhail.ink"
kind = "forward"

[[zones.suhail-ink.records]]
name = "nexus"
type = "host"
target = "nexus"
families = ["ipv4"]
```

An authoritative server answers for zones it owns. A recursive resolver looks up names on behalf of clients. Recursion ACLs should include only trusted networks.

## 9. Configure remote access

Create `ssh.toml`:

```toml
schema_version = 1

[clients.quasar.identities.default-linux]
file = "~/.ssh/id_rsa"

[clients.quasar.identities.proxmox]
file = "~/.ssh/id_proxmox"

[targets.nexus]
host = "nexus"
user = "suhail"
identity = "default-linux"

[targets.zion]
host = "zion"
user = "root"
identity = "proxmox"

[targets.atlas]
host = "atlas"
user = "root"
identity = "proxmox"
```

The SSH adapter emits include fragments and guarded install, verify, and rollback scripts. It does not own private keys.

## 10. Select external ingress

In the location file:

```toml
[external_ingress]
mode = "cloudflare-tunnel"
provider = "suhail-ink"
tunnel = "nexus-ingress"
```

Then create `cloudflare.toml`:

```toml
schema_version = 1

[providers.suhail-ink]
zone = "suhail.ink"
zone_id = "YOUR_ZONE_ID"
account_id = "YOUR_ACCOUNT_ID"
tunnel_api_token_env = "CLOUDFLARE_API_TOKEN_SUHAIL_INK_TUNNEL"
dns_api_token_env = "CLOUDFLARE_API_TOKEN_SUHAIL_INK_DNS"

[tunnels.nexus-ingress]
provider = "suhail-ink"
connector_host = "nexus"
origin = "https://127.0.0.1:443"
origin_tls_verify = false
hostnames = [
  "dsm.suhail.ink",
  "zion.suhail.ink",
]
```

Store token values outside Git. The TOML contains only environment-variable names. Separate tokens let the tunnel and DNS permissions remain narrowly scoped.

## 11. Validate before rendering

```bash
netweft validate
netweft adapters list
netweft show proxy
netweft show cloudflare
```

Validation checks references, address membership, duplicate ports, unsupported combinations, and adapter-specific invariants. Warnings deserve review even when they are not fatal.

## 12. Render one concern at a time

```bash
netweft render proxmox --host atlas
netweft render nginx --host nexus
netweft render ssh --client quasar
netweft render cloudflare --tunnel nexus-ingress
```

After every render:

```bash
find ~/.local/share/netweft/generated -type f | sort
```

Never infer the presence of `install.sh`, `verify.sh`, or `rollback.sh`. Use the actual rendered tree and the [adapter output table](../adapters/README.md).

## 13. Deploy safely

A safe deployment sequence is:

```text
validate
→ inspect resolved plan
→ render
→ inspect files
→ back up target state
→ stage artifacts
→ run provider-native validation
→ apply explicitly
→ verify
```

For remote network changes, retain console access because SSH can disconnect. In a Proxmox cluster, migrate Corosync addresses while quorum is healthy and never let two partitioned sides operate independently.

## Next references

- [Configuration option index](OPTION-INDEX.md)
- [Inventory](INVENTORY.md)
- [Locations](LOCATIONS.md)
- [Services](SERVICES.md)
- [DNS](DNS.md)
- [Deployment](../deployment/README.md)
