# Netweft

Deterministic network planning and configuration generation for portable infrastructure.

Netweft turns a small, typed TOML inventory into validated network plans and generated host artifacts. It is designed for homelabs and portable infrastructure that must survive location changes, ISP prefix changes, and host migrations without scattering deployment-specific addresses through source code.

Netweft `0.1.0` can:

- model hosts, logical networks, services, locations, DNS zones, and Tailscale routing;
- derive stable IPv6 ULA subnets and interface addresses;
- derive DNS recursion ACLs from the active location;
- build authoritative forward and reverse DNS plans;
- render a complete BIND configuration tree;
- render per-host environment files for Docker Compose, POSIX shells, Fish, and PowerShell;
- validate cross-file references, address ranges, duplicate ports, routing policies, and schema versions;
- keep generated runtime artifacts outside the source configuration tree.

Netweft plans and renders configuration. It does **not** deploy containers, alter router settings, approve Tailscale routes, or mutate host firewall rules.

## Installation

Install the latest published release:

```console
cargo install netweft
```

Install from a local checkout:

```console
cargo install --path . --force
```

Verify the installation:

```console
netweft --version
netweft --help
```

Netweft `0.1.0` is built with Rust 1.94 and edition 2024.

## Filesystem layout

By default Netweft follows XDG-style paths:

```text
~/.config/netweft/                 source configuration
~/.local/share/netweft/generated/  rendered artifacts
~/.local/state/netweft/            persistent state
~/.cache/netweft/                  cache data
```

Inspect the resolved paths:

```console
netweft paths
```

Override the configuration directory with either:

```console
netweft --config-dir /path/to/config validate
```

or:

```console
export NETWEFT_CONFIG_DIR=/path/to/config
```

`XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_STATE_HOME`, and `XDG_CACHE_HOME` are respected when present.

## Configuration layout

A configuration directory contains:

```text
~/.config/netweft/
├── netweft.toml
├── inventory.toml
├── networks.toml
├── services.toml
├── dns.toml
├── allocations.toml
└── locations/
    ├── home.toml
    └── travel.toml
```

All files in schema version 1 begin with:

```toml
schema_version = 1
```

Unknown fields are rejected. This catches misspellings and stale configuration instead of silently ignoring them.

## Minimal configuration walkthrough

The examples below use reserved documentation domains and addresses. Replace them with values appropriate for your network.

### 1. Select the active location

`netweft.toml`:

```toml
schema_version = 1
active_location = "home"

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

The active location determines which router, LAN segments, host addresses, and ISP IPv6 conditions are in effect.

Validate another location without changing the file:

```console
netweft validate --location travel
```

### 2. Define stable inventory

`inventory.toml`:

```toml
schema_version = 1

[domains]
primary = "example.net"
additional = ["example.org"]

[hosts.gateway]
kind = "physical"
roles = ["dns", "reverse-proxy", "subnet-router"]
runtime_root = "/var/lib/example/services/gateway"
ssh_user = "operator"
enabled = true

[hosts.storage]
kind = "nas"
roles = ["storage"]
ssh_user = "operator"
enabled = true

[hosts.laptop]
kind = "laptop"
roles = ["client"]
enabled = true
```

Supported host kinds are:

```text
physical
vm
laptop
workstation
nas
```

Inventory describes what a host *is*. Location files describe where it is attached and which addresses it currently uses.

### 3. Define logical networks

`networks.toml`:

```toml
schema_version = 1

[networks.gateway-containers]
kind = "docker"
owner = "gateway"
docker_name = "services"
ipv4_cidr = "10.40.0.0/16"
ipv4_gateway = "10.40.0.1"
allocation_key = "gateway-containers"
preferred_display_id = 121
ula_enabled = true
reverse_dns = true
dns_clients = true

[networks.gateway-containers.routing]
from = "gateway"
mode = "via-host"
via = "gateway"
```

Supported network kinds are `lan` and `docker`.

Routing modes are:

- `direct`: the network is directly reachable;
- `via-host`: the network is routed through a named host;
- `host-private`: the network must not be advertised externally.

`allocation_key` connects a logical network to its stable ULA subnet allocation.

Netweft `0.1.0` does not create Docker networks. Deployment tooling must create the external network from the rendered values. When a Docker bridge should accept routed container traffic from Tailscale, create it with a trusted interface, for example:

```console
docker network create \
  --driver bridge \
  --opt com.docker.network.bridge.trusted_host_interfaces=tailscale0 \
  --ipv6 \
  --subnet "$SERVICES_IPV4_SUBNET" \
  --gateway "$SERVICES_IPV4_GATEWAY" \
  --subnet "$SERVICES_IPV6_SUBNET" \
  --gateway "$SERVICES_IPV6_GATEWAY" \
  services
```

The interface name is deployment configuration; it is not hard-coded by Netweft.

### 4. Allocate a stable ULA

`allocations.toml`:

```toml
schema_version = 1

[ula]
prefix = "fd12:3456:789a::/48"

[ula.segments]
lan = 256

[ula.networks]
gateway-containers = 289
```

The numeric values become the fourth IPv6 hextet:

```text
segment 256  -> fd12:3456:789a:100::/64
network 289  -> fd12:3456:789a:121::/64
```

Use one randomly generated ULA `/48` for the deployment and keep it stable. Do not copy the example prefix into a real network.

### 5. Describe a location

`locations/home.toml`:

```toml
schema_version = 1
name = "home"
description = "Primary home network"

[router]
kind = "isp-gateway"
managed = false
supports_vlans = false

[ipv6]
mode = "router-advertised"
prefix = "2001:db8:100:200::/64"
subnet_prefix_length = 64
stability = "dynamic"
publish_public_aaaa = false

[segments.lan]
kind = "lan"
ipv4_cidr = "192.0.2.0/24"
ipv4_gateway = "192.0.2.1"
dns_clients = true
reverse_dns = true

[hosts.gateway.interfaces.lan]
segment = "lan"
ipv4 = "192.0.2.10"
ipv6_mode = "static"
ula_interface_id = "10"

[hosts.storage.interfaces.lan]
segment = "lan"
ipv4 = "192.0.2.20"
ipv6_mode = "static"
ula_interface_id = "20"

[tailscale]
enabled = true
strategy = "subnet-router"
primary_router = "gateway"

[tailscale.routers.gateway]
enabled = true
snat_subnet_routes = true
accept_routes = false
exit_node = false
advertise = [
  "segment:lan",
  "network:gateway-containers",
]
```

IPv6 location modes are:

- `disabled`;
- `router-advertised`, for a directly advertised `/64` such as an ISP gateway LAN;
- `delegated`, for a prefix Netweft may subdivide into `/64` networks.

A dynamic router-advertised prefix may be admitted to DNS recursion, but Netweft will not treat it as a durable allocation and will reject durable public AAAA publication from it.

### 6. Define services

`services.toml`:

```toml
schema_version = 1

[services.bind]
kind = "dns"
host = "gateway"
network = "gateway-containers"
enabled = true

[services.bind.address]
ipv4 = "10.40.20.20"
ipv6_interface_id = "2020"

[services.bind.ingress]
mode = "host-port"
interface = "lan"

[[services.bind.ports]]
host = 53
container = 53
protocol = "tcp"

[[services.bind.ports]]
host = 53
container = 53
protocol = "udp"

[services.proxy]
kind = "reverse-proxy"
host = "gateway"
network = "gateway-containers"
enabled = true

[services.proxy.address]
ipv4 = "10.40.80.80"
ipv6_interface_id = "8080"

[services.proxy.ingress]
mode = "host-port"
interface = "lan"

[[services.proxy.ports]]
host = 80
container = 80
protocol = "tcp"

[[services.proxy.ports]]
host = 443
container = 443
protocol = "tcp"
```

Service kinds are:

```text
dns
reverse-proxy
development-container
web
database
generic
```

A service may have:

- a stable container IPv4 address;
- a stable ULA interface ID;
- host-port ingress through one of the host’s location interfaces;
- published port mappings;
- optional SSH or web metadata.

### 7. Define DNS

`dns.toml`:

```toml
schema_version = 1

[dns]
provider = "bind9"
service = "bind"
default_ttl = 300
negative_ttl = 60

[dns.soa]
primary_nameserver = "ns1.example.net"
responsible_mailbox = "hostmaster.example.net"
refresh = 3600
retry = 900
expire = 1209600

[dns.recursion]
enabled = true
include_location_segments = true
include_tailscale = true
include_ula = true
include_docker_networks = true

[dns.forwarders]
ipv4 = ["1.1.1.1", "1.0.0.1"]
ipv6 = []

[[zones]]
name = "example.net"
visibility = "internal"
authoritative = true

[[records]]
name = "gateway.example.net"
kind = "host"
target = "gateway"
interface = "lan"
families = ["ipv4", "ipv6"]
reverse = true

[[records]]
name = "bind.internal.example.net"
kind = "service"
target = "bind"
address_scope = "container"
families = ["ipv4", "ipv6"]
reverse = true

[[records]]
name = "ns1.example.net"
kind = "service"
target = "bind"
address_scope = "ingress"
families = ["ipv4"]
reverse = false
```

DNS record kinds are:

- `host`: resolve a host interface;
- `service`: resolve a service container or ingress address;
- `proxy`: resolve the ingress address of a reverse-proxy service;
- `cname`: create a CNAME;
- `segment-gateway`: resolve a location segment gateway.

Address scopes for service records are `container` and `ingress`.

## Validate and inspect

Validate the active configuration:

```console
netweft validate
```

Validate another location:

```console
netweft validate --location travel
```

Inspect typed configuration:

```console
netweft show config
netweft show hosts
netweft show networks
netweft show services
```

Inspect derived plans:

```console
netweft show dns-access
netweft show dns
netweft show env --host gateway
```

Validation checks include:

- schema-version agreement;
- references between hosts, services, networks, locations, and DNS records;
- IPv4 addresses and gateways inside their configured networks;
- duplicate service IPv4 addresses;
- duplicate host ports;
- routing-policy consistency;
- Tailscale advertisement references;
- ULA allocation consistency;
- unsafe durable AAAA publication from dynamic router-advertised prefixes.

## Render artifacts

Render BIND:

```console
netweft render bind
```

Render one host’s environment files:

```console
netweft render env --host gateway
```

Render both:

```console
netweft render all --host gateway
```

The generated tree is location-specific:

```text
~/.local/share/netweft/generated/home/
├── bind/
│   ├── named.conf
│   ├── named.conf.options
│   ├── named.conf.local
│   ├── manifest.txt
│   └── zones/
│       └── db.*
└── hosts/
    └── gateway/
        ├── compose.env
        ├── shell.sh
        ├── shell.fish
        └── shell.ps1
```

Netweft also updates:

```text
~/.local/share/netweft/current
```

to point at the currently rendered location.

Generated files begin with a warning that they must not be edited manually.

## Docker Compose integration

Use the generated environment as the non-secret Compose environment:

```console
docker compose \
  --env-file "$HOME/.local/share/netweft/current/hosts/gateway/compose.env" \
  --env-file .env \
  config
```

Recommended split:

```text
Netweft compose.env  generated topology, addresses, routes, and paths
service .env         local secrets and credentials
```

Do not store authentication keys, API tokens, passwords, or private keys in Netweft configuration.

A Compose wrapper can make the split repeatable:

```sh
#!/bin/sh
set -eu

exec docker compose \
  --env-file "$HOME/.local/share/netweft/current/hosts/gateway/compose.env" \
  --env-file .env \
  "$@"
```

## Shell and Apogee integration

POSIX shells can source the generated environment directly:

```sh
. "$HOME/.local/share/netweft/current/hosts/gateway/shell.sh"
```

Fish:

```fish
source "$HOME/.local/share/netweft/current/hosts/gateway/shell.fish"
```

PowerShell:

```powershell
. "$HOME/.local/share/netweft/current/hosts/gateway/shell.ps1"
```

An environment manager such as Apogee can source the host file dynamically. Keep secrets in the environment manager’s secret store, not in generated Netweft files.

## Tailscale integration

For a host configured as a subnet router, Netweft renders:

```text
TAILSCALE_HOSTNAME
TAILSCALE_STATE_DIR
TS_ADVERTISE_ROUTES
```

The route list is derived from location segments and logical networks referenced by the router’s `advertise` entries.

Netweft does not:

- authenticate a Tailscale node;
- approve advertised routes in the Tailscale admin console;
- change client route-acceptance settings;
- edit tailnet grants or ACLs.

For stable internal DNS over Tailscale, publish an owned ULA container address and configure split DNS for only the private zones that should use it.

## BIND behavior

The generated BIND tree includes:

- authoritative forward zones;
- generated IPv4 and IPv6 reverse zones;
- recursion ACLs derived from the active location;
- configured forwarders;
- disabled dynamic updates and zone transfers;
- deterministic record ordering.

In `0.1.0`, generated SOA serials are currently fixed at `1`. A future release will add durable monotonic serial management.

## Security model

Netweft deliberately separates topology from secrets.

Safe Netweft inputs include:

- host and service names;
- network CIDRs and stable addresses;
- ULA allocations;
- ports and routing policy;
- DNS zones and records;
- generated-output paths.

Keep these elsewhere:

- Tailscale auth keys;
- Cloudflare or DNS-provider API tokens;
- database passwords;
- TLS private keys;
- SSH private keys;
- application credentials.

Avoid committing real deployment configuration to the public Netweft source repository. The Rust source must not contain private hostnames, domains, addresses, interface names, or service names.

## Current limitations

Netweft `0.1.0`:

- renders configuration but does not deploy it;
- does not create Docker networks or containers;
- does not configure routers, DHCP, VLANs, or firewalls;
- does not mutate Tailscale control-plane settings;
- does not discover live infrastructure automatically;
- uses a fixed BIND SOA serial;
- supports only `/64` subdivision from delegated IPv6 prefixes;
- renders one active location at a time.

These boundaries are intentional: the first release focuses on a small, deterministic, reviewable planning core.

## Development

```console
cargo fmt --all -- --check
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

Inspect the package contents:

```console
cargo package --list
cargo package
```

Test a local installation:

```console
cargo install --path . --force
netweft --version
netweft --help
```

## Release checklist

```console
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo package --list
cargo package
cargo publish --dry-run
```

Then commit, tag, push, and publish the same source revision.

## License

Licensed under the MIT License.
