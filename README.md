# Netweft

Deterministic network planning and configuration generation for portable infrastructure.

Netweft keeps stable infrastructure intent in TOML, validates relationships between hosts, networks, services, locations, DNS, IPv6 allocation, and Tailscale routing, then renders deployment artifacts such as:

- complete BIND configuration;
- shell environment files;
- Docker Compose environment files;
- host-specific generated state.

Netweft does **not** directly modify routers, start containers, change operating-system networking, or deploy generated files in version `0.1.0`. It plans and renders; deployment remains explicit.

---

## Why Netweft exists

A home lab often begins with values scattered across:

- Docker Compose files;
- BIND zone files;
- shell environment variables;
- Tailscale startup commands;
- router reservations;
- host-specific scripts;
- copied configuration from an older location.

That works until something changes:

- the physical location changes;
- an ISP delegates a different IPv6 prefix;
- a host receives a new address;
- a Docker network is recreated;
- a service moves to another machine;
- the same machine must operate at multiple sites.

Netweft provides one typed configuration model for those relationships.

The central rule is:

> Deployment-specific names, addresses, domains, network names, and service names belong in configuration, not in Rust source code.

---

## Current production example

The examples in this README follow the current `shane-xfinity` deployment.

### Physical network

| Item | Value |
|---|---|
| Router | Xfinity XB8 |
| Main LAN | `10.214.90.0/24` |
| Router | `10.214.90.1` |
| Nexus | `10.214.90.10` |
| DS1621+ | `10.214.90.20` |
| ISP IPv6 mode | Router-advertised |
| ISP IPv6 prefix | Dynamic |
| Netweft ULA | `fdd7:d134:61e3::/48` |

### Nexus services

| Service | IPv4 | Stable ULA |
|---|---:|---:|
| BIND | `10.78.29.29` | `fdd7:d134:61e3:121::2929` |
| Nginx Proxy Manager | `10.78.88.88` | `fdd7:d134:61e3:121::8888` |
| MariaDB | `10.78.29.31` | `fdd7:d134:61e3:121::2931` |

### Friendly names

| Name | Purpose |
|---|---|
| `nexus.suhail.ink` | Nexus host |
| `nginx.suhail.ink` | Nginx Proxy Manager ingress |
| `nginx.internal.suhail.ink` | Nginx container address |
| `bind9.internal.suhail.ink` | BIND container address |
| `ds1621plus.suhail.ink` | Synology DS1621+ |
| `ns1.suhail.ink` | Internal nameserver |

---

## Installation

### From crates.io

```bash
cargo install netweft
```

### From a source checkout

```bash
cargo install --path . --force
```

Confirm:

```bash
netweft --version
netweft --help
```

Expected:

```text
netweft 0.1.0
```

Netweft uses Rust edition 2024 and has been tested with Rust 1.94.

---

## Filesystem paths

Run:

```bash
netweft paths
```

Default paths:

```text
Config:    ~/.config/netweft
Generated: ~/.local/share/netweft/generated
State:     ~/.local/state/netweft
Cache:     ~/.cache/netweft
```

Environment overrides:

| Variable | Meaning |
|---|---|
| `NETWEFT_CONFIG_DIR` | Source configuration directory |
| `XDG_CONFIG_HOME` | Parent of the default config directory |
| `XDG_DATA_HOME` | Parent of generated output |
| `XDG_STATE_HOME` | Parent of state |
| `XDG_CACHE_HOME` | Parent of cache |

A command-line override is also available:

```bash
netweft --config-dir /path/to/config validate
```

---

## Configuration directory

A complete configuration contains:

```text
~/.config/netweft/
├── netweft.toml
├── inventory.toml
├── networks.toml
├── services.toml
├── dns.toml
├── allocations.toml
└── locations/
    ├── shane-xfinity.toml
    └── la-unifi.toml
```

Each file has one responsibility.

| File | Responsibility |
|---|---|
| `netweft.toml` | Global settings and active location |
| `inventory.toml` | Stable host identities |
| `networks.toml` | Stable logical networks |
| `services.toml` | Services and their placement |
| `dns.toml` | Zones, records, forwarders, recursion policy |
| `allocations.toml` | Stable ULA allocation IDs |
| `locations/*.toml` | Site-specific addressing and routing |

All files currently use:

```toml
schema_version = 1
```

Unknown fields are rejected. This is intentional: a misspelled key must fail validation rather than be silently ignored.

---

# Configuration reference

## `netweft.toml`

This file selects the active location and controls rendering and validation behavior.

```toml
schema_version = 1
active_location = "shane-xfinity"

[paths]
# Optional. XDG defaults are used when omitted.
# generated_root = "/custom/generated/path"
# state_root = "/custom/state/path"
# cache_root = "/custom/cache/path"

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

### Important fields

- `active_location`: location loaded unless `--location` overrides it.
- `atomic`: render into temporary paths and replace output as a unit.
- `stable_order`: emit deterministic ordering.
- `generated_headers`: add generated-file warnings.
- `deny_warnings`: turn validation warnings into errors.

Override the location for one command:

```bash
netweft validate --location la-unifi
```

---

## `inventory.toml`

Inventory defines stable machine identities. It should not contain site-specific IP addresses.

```toml
schema_version = 1

[domains]
primary = "suhail.ink"
additional = [
    "suhail.photos",
    "suhail.art",
    "suhail.life",
]

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

[hosts.eclipse]
kind = "laptop"
roles = ["client", "development"]
enabled = true

[hosts.quasar]
kind = "workstation"
roles = ["client", "development"]
enabled = true

[hosts.zion]
kind = "physical"
roles = ["proxmox"]
ssh_user = "suhail"
enabled = true

[hosts.atlas]
kind = "physical"
roles = ["proxmox"]
ssh_user = "suhail"
enabled = true

[hosts.summit]
kind = "physical"
roles = ["proxmox"]
ssh_user = "suhail"
enabled = false

[hosts.vortex]
kind = "vm"
parent = "zion"
roles = ["docker", "gpu", "development"]
runtime_root = "/var/lib/suhail/services/vortex"
ssh_user = "suhail"
enabled = true
```

Supported host kinds:

```text
physical
vm
laptop
workstation
nas
```

### Host fields

| Field | Required | Meaning |
|---|---:|---|
| `kind` | yes | Stable machine type |
| `roles` | no | Descriptive capabilities |
| `parent` | no | Parent host for a VM |
| `runtime_root` | no | Host-local service state root |
| `ssh_user` | no | Default SSH user |
| `enabled` | no | Defaults to `true` |

### Naming guidance

Use stable, specific identities:

```text
ds1621plus
```

rather than generic vendor names:

```text
synology
```

This makes room for additional NAS devices later.

---

## `networks.toml`

Networks describe stable logical networks independent of any one location.

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
dns_clients = true

[networks.nexus-containers.routing]
from = "nexus"
mode = "direct"

[networks.vortex-containers]
kind = "docker"
owner = "vortex"
docker_name = "vortex"
ipv4_cidr = "10.74.0.0/16"
ipv4_gateway = "10.74.0.1"
allocation_key = "vortex-containers"
preferred_display_id = 321
ula_enabled = true
reverse_dns = true
dns_clients = true

[networks.vortex-containers.routing]
from = "nexus"
mode = "via-host"
via = "vortex"
```

Supported network kinds:

```text
lan
docker
```

Supported routing modes:

| Mode | Meaning |
|---|---|
| `direct` | Directly reachable from the named host |
| `via-host` | Reachable through another host |
| `host-private` | Private to the owner host |

### Why routing matters

Netweft derives DNS recursion ACLs from reachability.

For example, `nexus-containers` is directly reachable by BIND on Nexus, so `10.78.0.0/16` may use recursion.

`vortex-containers` is not currently directly reachable from Nexus, so Netweft warns and excludes `10.74.0.0/16`.

---

## `services.toml`

Services define placement, network attachment, container addresses, ports, ingress, and optional web behavior.

A representative Nexus configuration:

```toml
schema_version = 1

[services.bind9]
kind = "dns"
host = "nexus"
network = "nexus-containers"
enabled = true

[services.bind9.address]
ipv4 = "10.78.29.29"
ipv6_interface_id = "2929"

[[services.bind9.ports]]
host = 53
container = 53
protocol = "tcp"

[[services.bind9.ports]]
host = 53
container = 53
protocol = "udp"

[services.bind9.ingress]
mode = "host-port"
interface = "lan"

[services.nginx]
kind = "reverse-proxy"
host = "nexus"
network = "nexus-containers"
enabled = true

[services.nginx.address]
ipv4 = "10.78.88.88"
ipv6_interface_id = "8888"

[[services.nginx.ports]]
host = 80
container = 80
protocol = "tcp"

[[services.nginx.ports]]
host = 81
container = 81
protocol = "tcp"

[[services.nginx.ports]]
host = 443
container = 443
protocol = "tcp"

[services.nginx.ingress]
mode = "host-port"
interface = "lan"

[services.db]
kind = "database"
host = "nexus"
network = "nexus-containers"
enabled = true

[services.db.address]
ipv4 = "10.78.29.31"
ipv6_interface_id = "2931"
```

Supported service kinds:

```text
dns
reverse-proxy
development-container
web
database
generic
```

### Service address scopes

DNS records can resolve either:

- the service’s container address;
- the host ingress address.

Example:

```toml
address_scope = "container"
```

resolves Nginx to:

```text
10.78.88.88
fdd7:d134:61e3:121::8888
```

Example:

```toml
address_scope = "ingress"
```

resolves Nginx to Nexus:

```text
10.214.90.10
```

This distinction is useful for keeping both:

```text
nginx.internal.suhail.ink
nginx.suhail.ink
```

---

## `allocations.toml`

This file makes ULA allocation durable.

```toml
schema_version = 1

[ula]
prefix = "fdd7:d134:61e3::/48"

[ula.segments]
main = 256

[ula.networks]
nexus-containers = 289
vortex-containers = 321
```

The decimal allocation IDs are rendered as hexadecimal subnet IDs:

| Key | Decimal | Hex | Result |
|---|---:|---:|---|
| `main` | `256` | `0x100` | `fdd7:d134:61e3:100::/64` |
| `nexus-containers` | `289` | `0x121` | `fdd7:d134:61e3:121::/64` |
| `vortex-containers` | `321` | `0x141` | `fdd7:d134:61e3:141::/64` |

Do not casually change an existing allocation ID after deployment. That would renumber stable ULA addresses.

---

## `locations/shane-xfinity.toml`

A location attaches stable inventory to a physical site.

```toml
schema_version = 1
name = "shane-xfinity"
description = "Shane's Xfinity XB8 network in San Francisco"

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

[hosts.eclipse.interfaces.wifi]
segment = "main"
ipv6_mode = "slaac"

[tailscale]
enabled = true
strategy = "subnet-router"
primary_router = "nexus"

[tailscale.routers.nexus]
enabled = true
snat_subnet_routes = true
accept_routes = false
exit_node = false
advertise = [
    "segment:main",
    "network:nexus-containers",
]
```

### IPv6 modes

| Mode | Meaning |
|---|---|
| `disabled` | No location IPv6 |
| `router-advertised` | Prefix learned from the router |
| `delegated` | Prefix delegated and allocatable |

For Xfinity:

```toml
mode = "router-advertised"
stability = "dynamic"
publish_public_aaaa = false
```

This means Netweft:

- permits the live prefix in DNS recursion ACLs;
- does not treat it as durable allocation space;
- does not generate stable GUA AAAA records from it.

### Host interface IPv6 modes

```text
slaac
static
disabled
```

`ula_interface_id` is a stable identifier used when deriving a ULA, but an AAAA record should only be published when the host actually owns and can answer on that ULA.

The Synology currently remains IPv4-only in private DNS because DSM uses Xfinity automatic IPv6 and does not own the proposed Netweft ULA.

---

## `dns.toml`

This file defines BIND behavior, zones, records, recursion ACLs, and forwarding.

```toml
schema_version = 1

[dns]
provider = "bind9"
service = "bind9"
default_ttl = 86400
negative_ttl = 604800

[dns.soa]
primary_nameserver = "ns1.suhail.ink."
responsible_mailbox = "admin.suhail.ink."
refresh = 86400
retry = 7200
expire = 3600000

[dns.recursion]
enabled = true
include_location_segments = true
include_tailscale = true
include_ula = true
include_docker_networks = true

[dns.forwarders]
ipv4 = ["1.1.1.1", "1.0.0.1"]
ipv6 = [
    "2606:4700:4700::1111",
    "2606:4700:4700::1001",
]

[[zones]]
name = "suhail.ink"
visibility = "internal"
authoritative = true

[[zones]]
name = "suhail.photos"
visibility = "internal"
authoritative = true

[[zones]]
name = "suhail.art"
visibility = "internal"
authoritative = true

[[zones]]
name = "suhail.life"
visibility = "internal"
authoritative = true
```

### Canonical host record

```toml
[[records]]
name = "nexus.suhail.ink"
kind = "host"
target = "nexus"
interface = "lan"
families = ["ipv4"]
reverse = true
```

### Container service record

```toml
[[records]]
name = "bind9.internal.suhail.ink"
kind = "service"
target = "bind9"
address_scope = "container"
families = ["ipv4", "ipv6"]
reverse = true
```

### Friendly ingress record

```toml
[[records]]
name = "nginx.suhail.ink"
kind = "service"
target = "nginx"
address_scope = "ingress"
families = ["ipv4"]
reverse = false
```

This resolves to the host ingress address:

```text
nginx.suhail.ink -> 10.214.90.10
```

The Nginx Proxy Manager administration interface remains on port `81` unless a proxy host forwards standard HTTPS to it:

```text
http://nginx.suhail.ink:81
```

### Internal container record

```toml
[[records]]
name = "nginx.internal.suhail.ink"
kind = "service"
target = "nginx"
address_scope = "container"
families = ["ipv4", "ipv6"]
reverse = true
```

### DS1621+ record

```toml
[[records]]
name = "ds1621plus.suhail.ink"
kind = "host"
target = "ds1621plus"
interface = "lan"
families = ["ipv4"]
reverse = true
```

Result:

```text
ds1621plus.suhail.ink.  A    10.214.90.20
10.214.90.20            PTR  ds1621plus.suhail.ink.
```

No AAAA record is emitted because the NAS does not currently own a durable Netweft ULA.

### Record kinds

```text
host
service
proxy
cname
segment-gateway
```

### DNS visibility

```text
internal
public
both
```

---

# Commands

## Validate

```bash
netweft validate
```

Override the active location:

```bash
netweft validate --location shane-xfinity
```

Validation checks relationships such as:

- referenced hosts exist;
- referenced networks exist;
- services have valid owners and networks;
- interface addresses belong to their segments;
- ULA allocation IDs are valid;
- DNS targets are resolvable;
- recursion networks are reachable;
- dynamic IPv6 is not treated as stable allocation space.

---

## Show configuration

```bash
netweft show config
netweft show hosts
netweft show networks
netweft show services
```

Show automatically derived recursion clients:

```bash
netweft show dns-access
```

Show the resolved DNS plan without writing files:

```bash
netweft show dns
```

Show a host environment plan:

```bash
netweft show env --host nexus
```

---

## Render BIND

```bash
netweft render bind
```

Default output:

```text
~/.local/share/netweft/generated/<location>/bind/
```

Example:

```text
~/.local/share/netweft/generated/shane-xfinity/bind/
├── named.conf
├── named.conf.local
├── named.conf.options
├── manifest.txt
└── zones/
    ├── db.suhail.ink
    ├── db.suhail.life
    ├── db.suhail.photos
    ├── db.suhail.art
    ├── db.90.214.10.in-addr.arpa
    ├── db.29.78.10.in-addr.arpa
    ├── db.88.78.10.in-addr.arpa
    └── db.<ipv6-reverse-zone>
```

Inspect a generated record:

```bash
grep -R -n 'ds1621plus.suhail.ink' \
  ~/.local/share/netweft/generated/shane-xfinity/bind
```

---

## Render host environment

```bash
netweft render env --host nexus
```

Render all host artifacts:

```bash
netweft render all --host nexus
```

Default output:

```text
~/.local/share/netweft/generated/<location>/hosts/<host>/
```

For Nexus:

```text
~/.local/share/netweft/generated/shane-xfinity/hosts/nexus/
├── compose.env
├── shell.sh
├── shell.fish
└── shell.ps1
```

Representative generated values:

```text
NETWEFT_LOCATION=shane-xfinity
NETWEFT_GENERATED_ROOT=/home/suhail/.local/share/netweft/generated/shane-xfinity
NETWEFT_BIND_CONFIG_DIR=/home/suhail/.local/share/netweft/generated/shane-xfinity/bind

NEXUS_RUNTIME=/var/lib/suhail/services/nexus
NEXUS_LAN_IPV4=10.214.90.10
NEXUS_LAN_IPV6=fdd7:d134:61e3:100::10

FUJI_IPV4_SUBNET=10.78.0.0/16
FUJI_IPV4_GATEWAY=10.78.0.1
FUJI_IPV6_SUBNET=fdd7:d134:61e3:121::/64
FUJI_IPV6_GATEWAY=fdd7:d134:61e3:121::1

BIND9_IPV4=10.78.29.29
BIND9_IPV6=fdd7:d134:61e3:121::2929

NGINX_IPV4=10.78.88.88
NGINX_IPV6=fdd7:d134:61e3:121::8888

DB_IPV4=10.78.29.31
DB_IPV6=fdd7:d134:61e3:121::2931

TAILSCALE_HOSTNAME=nexus
TAILSCALE_STATE_DIR=/var/lib/suhail/services/nexus/tailscale/state
TS_ADVERTISE_ROUTES=10.214.90.0/24,10.78.0.0/16,fdd7:d134:61e3:100::10/128,fdd7:d134:61e3:121::/64
```

---

# Integration

## Apogee shell integration

Apogee can source the generated host environment.

Example module:

```toml
[modules.apps.netweft]
enabled = true
kind = "cli"
priority = 22
platforms = ["mac", "linux", "wsl", "other"]

[modules.apps.netweft.detect.commands]
any_of = ["netweft"]

[modules.apps.netweft.emit.env]
NETWEFT_CONFIG_DIR = "{xdg_config_home}/netweft"
NETWEFT_DATA_DIR = "{xdg_data_home}/netweft"
NETWEFT_STATE_DIR = "{xdg_state_home}/netweft"

[modules.apps.netweft.emit.source]
files = [
    "{xdg_data_home}/netweft/current/hosts/{host}/shell.{shell_family_ext}",
]
```

On Bash:

```bash
eval "$(APOGEE_SHELL=bash apogee)"
```

Generated infrastructure variables become available in every shell without manually duplicating values.

Secrets remain in Apogee’s secret environment, not in Netweft configuration.

---

## Docker Compose integration

A Compose wrapper can load:

1. Netweft-generated topology;
2. host-local service secrets.

Example:

```bash
docker compose \
  --env-file "$HOME/.local/share/netweft/current/hosts/nexus/compose.env" \
  --env-file .env \
  "$@"
```

Recommended separation:

| File | Content |
|---|---|
| generated `compose.env` | addresses, paths, names, routes |
| local `.env` | credentials and secrets |

Never commit:

```text
TS_AUTHKEY
Cloudflare API token
database passwords
private keys
```

Be aware that:

```bash
docker compose config
```

prints resolved values and may expose secrets.

---

## BIND deployment

The live BIND container mounts generated configuration read-only:

```yaml
services:
  bind9:
    image: internetsystemsconsortium/bind9:9.20
    volumes:
      - ${NETWEFT_BIND_CONFIG_DIR}:/etc/bind:ro
```

After rendering:

```bash
netweft render bind
```

validate and recreate BIND:

```bash
docker compose up -d --force-recreate --no-deps bind9
```

Test:

```bash
dig +short @127.0.0.1 nexus.suhail.ink A
dig +short @127.0.0.1 ds1621plus.suhail.ink A
dig +short @127.0.0.1 -x 10.214.90.20
```

---

## Docker IPv6 direct routing through Tailscale

Docker bridge networks protect direct access to container addresses using raw-table rules.

For direct Tailscale access to published container ports, create the external bridge with a trusted host interface:

```bash
docker network create \
  --driver bridge \
  --opt com.docker.network.bridge.trusted_host_interfaces=tailscale0 \
  --ipv6 \
  --subnet 10.78.0.0/16 \
  --gateway 10.78.0.1 \
  --subnet fdd7:d134:61e3:121::/64 \
  --gateway fdd7:d134:61e3:121::1 \
  fuji
```

This causes Docker to install explicit allow rules for `tailscale0` before its direct-routing drop rules.

Verify:

```bash
docker network inspect fuji \
  --format 'Options={{json .Options}} IPAM={{json .IPAM.Config}}'
```

Expected:

```text
"com.docker.network.bridge.trusted_host_interfaces":"tailscale0"
```

This network creation remains deployment responsibility in Netweft `0.1.0`.

---

## Tailscale

Nexus acts as a subnet router and advertises selected location segments and logical networks.

The generated environment includes `TS_ADVERTISE_ROUTES`.

Routes must also be approved in the Tailscale admin console.

### Split DNS

For clients such as Eclipse, configure a restricted Tailscale nameserver:

```text
Nameserver:
fdd7:d134:61e3:121::2929

Restricted domains:
suhail.ink
suhail.life
suhail.photos
suhail.art
```

Then applications using the macOS system resolver can resolve private names through Nexus BIND.

On macOS, both must be enabled:

- Use Tailscale subnets
- Use Tailscale DNS settings

The standalone macOS Tailscale variant provides fuller CLI and system integration than the App Store variant.

### Synology

The DS1621+ also runs Tailscale directly.

Observed stable overlay addresses:

```text
100.105.105.85
fd7a:115c:a1e0::8835:6956
```

Tailnet domain:

```text
zapus-cod.ts.net
```

The primary friendly LAN name remains:

```text
ds1621plus.suhail.ink -> 10.214.90.20
```

This works:

- directly on the home LAN;
- remotely through Nexus’s subnet route;
- independently through the NAS’s direct Tailscale identity.

---

# Common workflow

Use this sequence after any configuration change:

```bash
netweft validate --location shane-xfinity
netweft show dns
netweft show env --host nexus
netweft render all --host nexus
```

Inspect generated output:

```bash
grep -R -n 'expected-name-or-address' \
  ~/.local/share/netweft/generated/shane-xfinity
```

Deploy only after the plan is correct.

For BIND:

```bash
docker compose up -d --force-recreate --no-deps bind9
```

Test forward and reverse records:

```bash
dig +short @127.0.0.1 ds1621plus.suhail.ink A
dig +short @127.0.0.1 -x 10.214.90.20
```

---

# Adding a new host

This example adds the DS1621+ NAS.

## 1. Reserve its IPv4 address

In the router, reserve:

```text
10.214.90.20
```

Verify:

```bash
ping -c 3 10.214.90.20
curl -kI https://10.214.90.20:5001/
```

## 2. Add inventory identity

`inventory.toml`:

```toml
[hosts.ds1621plus]
kind = "nas"
roles = ["storage"]
enabled = true
```

## 3. Attach it to the location

`locations/shane-xfinity.toml`:

```toml
[hosts.ds1621plus.interfaces.lan]
segment = "main"
ipv4 = "10.214.90.20"
ipv6_mode = "slaac"
ula_interface_id = "20"
```

## 4. Add DNS

`dns.toml`:

```toml
[[records]]
name = "ds1621plus.suhail.ink"
kind = "host"
target = "ds1621plus"
interface = "lan"
families = ["ipv4"]
reverse = true
```

## 5. Validate and render

```bash
netweft validate --location shane-xfinity
netweft show dns
netweft render bind
```

## 6. Deploy BIND

```bash
docker compose up -d --force-recreate --no-deps bind9
```

## 7. Verify

```bash
dig +short @127.0.0.1 ds1621plus.suhail.ink A
dig +short @127.0.0.1 -x 10.214.90.20
```

Expected:

```text
10.214.90.20
ds1621plus.suhail.ink.
```

---

# Adding a new service

Suppose a new web service runs on Nexus.

## 1. Add the service

`services.toml`:

```toml
[services.example]
kind = "web"
host = "nexus"
network = "nexus-containers"
enabled = true

[services.example.address]
ipv4 = "10.78.40.40"
ipv6_interface_id = "4040"

[services.example.web]
container_port = 8080
domain = "example.suhail.ink"
access = "reverse-proxy"
proxy = "nginx"
tls = true
```

## 2. Validate

```bash
netweft validate
netweft show services
netweft show dns
```

A web service with:

```toml
access = "reverse-proxy"
```

automatically gets a DNS ingress record at its configured domain through the selected proxy.

## 3. Render

```bash
netweft render all --host nexus
```

## 4. Deploy the container and configure Nginx Proxy Manager

The application container should listen on the configured network address and port.

Create the matching proxy host in Nginx Proxy Manager.

---

# Adding a friendly alias

To keep a canonical host record while adding a shorter alias:

```toml
[[records]]
name = "ds1621.suhail.ink"
kind = "host"
target = "ds1621plus"
interface = "lan"
families = ["ipv4"]
reverse = false
```

Keep only one reverse record:

```text
10.214.90.20 -> ds1621plus.suhail.ink
```

---

# Troubleshooting

## `dig` works but browser, `ping`, or `curl` cannot resolve

`dig` can query DNS directly while normal applications use the operating-system resolver.

Check:

```bash
dscacheutil -q host -a name nginx.suhail.ink
scutil --dns
```

On Tailscale clients, enable:

- Tailscale DNS settings;
- Tailscale subnet routes.

Flush macOS DNS:

```bash
sudo dscacheutil -flushcache
sudo killall -HUP mDNSResponder
```

---

## `http://nginx.suhail.ink` shows the congratulations page

That is port `80`, Nginx Proxy Manager’s public proxy listener.

The admin UI is:

```text
http://nginx.suhail.ink:81
```

To remove `:81`, configure an HTTPS proxy host for `nginx.suhail.ink` that forwards internally to `127.0.0.1:81`.

---

## DNS container works locally but not over Tailscale ULA

Check that the Docker network was created with:

```text
com.docker.network.bridge.trusted_host_interfaces=tailscale0
```

Inspect raw rules:

```bash
sudo ip6tables-save -t raw
```

Docker should place a `tailscale0` accept rule before the non-bridge drop rule.

---

## Docker is slow

Check disk space immediately:

```bash
df -h /
docker system df
```

Nexus previously reached 100% disk use because:

```text
~/Dropbox/.dropbox.cache
```

grew to approximately `177G`.

Do not assume Docker itself is the cause when `/` is full.

---

## Dynamic Xfinity IPv6

The Xfinity `/64` is useful for current connectivity but is not treated as a stable identity.

Do not publish durable AAAA records from:

```text
2601:645:8a02:6610::/64
```

Use:

- stable Netweft ULA for controlled infrastructure;
- Tailscale addresses for overlay identity;
- IPv4 LAN records where the device cannot own a stable ULA.

---

# Security

## Never store secrets in Netweft

Do not put these in TOML:

```text
Tailscale auth keys
Cloudflare API tokens
database passwords
private keys
certificate credentials
```

Use a secret manager, protected `.env`, or Apogee secret environment.

## Generated files are host-local

Recommended:

```text
~/.local/share/netweft/generated
```

Do not commit generated output containing environment-specific paths unless intentionally publishing an example.

## Be careful with logs

Nginx Proxy Manager startup logs and:

```bash
docker compose config
```

may print secret values.

Redact before sharing.

---

# Development

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

Check the package:

```bash
cargo package --list
cargo package
cargo publish --dry-run
```

Install locally:

```bash
cargo install --path . --force
```

---

# Release checklist

```bash
git status --short
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo package
cargo publish --dry-run
```

Then:

```bash
git add .
git commit -m "release netweft 0.1.0"
git tag -a v0.1.0 -m "Netweft 0.1.0"
git push origin main
git push origin v0.1.0
cargo publish
```

Never paste the crates.io token into chat or command history.

---

# Current limitations

Netweft `0.1.0`:

- renders but does not deploy;
- does not configure routers;
- does not create Docker networks;
- does not manage firewall persistence;
- does not approve Tailscale routes;
- does not modify Nginx Proxy Manager;
- does not install addresses on operating-system interfaces;
- does not yet maintain durable incrementing SOA serial state;
- treats router-advertised dynamic IPv6 as reachable but not allocatable.

These boundaries are deliberate. Generated plans remain inspectable before anything changes on a live system.

---

# Design principles

1. **Configuration owns deployment identity.**
2. **Rust source remains environment-agnostic.**
3. **Stable identities are separate from location attachment.**
4. **Dynamic ISP prefixes are not durable host identity.**
5. **Generated artifacts are deterministic and reviewable.**
6. **Secrets remain outside the configuration graph.**
7. **Deployment remains explicit.**
8. **Validation should fail early instead of silently guessing.**

---

# License

MIT
