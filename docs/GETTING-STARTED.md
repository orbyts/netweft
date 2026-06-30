# Getting started

This guide builds a Netweft configuration from an empty directory. The detailed reference for every TOML key is under [Configuration reference](configuration/README.md).

## 1. Create the configuration directory

```bash
mkdir -p ~/.config/netweft/locations
```

Required files:

```text
~/.config/netweft/
├── netweft.toml
├── inventory.toml
├── networks.toml
├── services.toml
├── dns.toml
├── allocations.toml
└── locations/
    └── <location>.toml
```

Optional files enable additional planners and adapters:

```text
docker.toml
ssh.toml
cloudflare.toml
guests.toml
mounts.toml
nas-permissions.toml
proxmox-storages.toml
proxmox-sdn.toml
```

## 2. Configure Netweft itself

```toml
# ~/.config/netweft/netweft.toml
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

See [netweft.toml](configuration/NETWEFT.md).

## 3. Define stable host identities

```toml
# inventory.toml
schema_version = 1

[domains]
primary = "example.net"
additional = []

[hosts.gateway]
kind = "physical"
roles = ["infrastructure", "dns", "proxy", "docker"]
runtime_root = "/var/lib/example/gateway"
ssh_user = "admin"
enabled = true
```

Host identity belongs here; site-specific IP addresses do not. See [inventory.toml](configuration/INVENTORY.md).

## 4. Define logical networks

```toml
# networks.toml
schema_version = 1

[networks.gateway-containers]
kind = "docker"
owner = "gateway"
docker_name = "gateway"
ipv4_cidr = "10.78.0.0/16"
ipv4_gateway = "10.78.0.1"
allocation_key = "gateway-containers"
ula_enabled = true
reverse_dns = true
dns_clients = true

[networks.gateway-containers.routing]
from = "gateway"
mode = "direct"
```

See [networks.toml](configuration/NETWORKS.md).

## 5. Attach hosts to a location

```toml
# locations/home.toml
schema_version = 1
name = "home"
description = "Primary home network"

[router]
kind = "generic"
managed = false
supports_vlans = false

[ipv6]
mode = "disabled"
stability = "dynamic"
publish_public_aaaa = false

[segments.main]
kind = "lan"
ipv4_cidr = "192.168.1.0/24"
ipv4_gateway = "192.168.1.1"
dns_clients = true
reverse_dns = true

[hosts.gateway.interfaces.lan]
segment = "main"
ipv4 = "192.168.1.10"
ipv6_mode = "disabled"
```

See [location files](configuration/LOCATIONS.md).

## 6. Define services

```toml
# services.toml
schema_version = 1

[services.bind9]
kind = "dns"
host = "gateway"
network = "gateway-containers"
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
```

See [services.toml](configuration/SERVICES.md).

## 7. Configure DNS

```toml
# dns.toml
schema_version = 1

[dns]
provider = "bind9"
service = "bind9"
default_ttl = 86400
negative_ttl = 604800

[dns.soa]
primary_nameserver = "ns1.example.net."
responsible_mailbox = "admin.example.net."
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
families = ["ipv4"]
reverse = true
```

See [dns.toml](configuration/DNS.md).

## 8. Configure ULA allocation

```toml
# allocations.toml
schema_version = 1

[ula]
prefix = "fd12:3456:789a::/48"

[ula.segments]
main = 256

[ula.networks]
gateway-containers = 289
```

See [allocations.toml](configuration/ALLOCATIONS.md).

## 9. Validate before rendering

```bash
netweft validate
netweft show config
netweft show hosts
netweft show networks
netweft show services
netweft show dns
```

## 10. Render explicitly

```bash
netweft render bind
netweft render env --host gateway
```

Rendering does not deploy. Use the matching guide under [Deployment](deployment/README.md).
