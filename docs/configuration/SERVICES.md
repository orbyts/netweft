# `services.toml`

Defines services, certificate references, placement, addresses, ports, SSH endpoints, and web proxy intent.

## Certificates

```toml
[certificates.wildcard-suhail]
domains = ["suhail.ink", "*.suhail.ink"]
certificate_path = "/etc/netweft/certificates/wildcard-suhail/fullchain.pem"
private_key_path = "/etc/netweft/certificates/wildcard-suhail/privkey.pem"
```

### `[certificates.<certificate>]`

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `domains` | `array<string>` | no | `[]` | Domain names covered by the certificate. Wildcards may be used where supported by validation. |
| `certificate_path` | `string` | yes | — | Absolute runtime path to the certificate/full chain as seen by the proxy. |
| `private_key_path` | `string` | yes | — | Absolute runtime path to the private key as seen by the proxy. |

Netweft stores references only; it does not read, issue, or renew certificate material.

## Services

```toml
[services.example]
kind = "web"
host = "nexus"
network = "nexus-containers"
enabled = true

[services.example.address]
ipv4 = "10.78.40.40"
ipv6_interface_id = "4040"

[[services.example.ports]]
host = 8080
container = 8080
protocol = "tcp"

[services.example.ingress]
mode = "host-port"
interface = "lan"

[services.example.web]
container_port = 8080
domain = "example.suhail.ink"
access = "reverse-proxy"
proxy = "nginx"
scheme = "http"
tls = true
certificate = "wildcard-suhail"
force_https = true
websocket = false
```

### `[services.<service>]`

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `kind` | enum | yes | — | `dns`, `reverse-proxy`, `development-container`, `web`, `database`, or `generic`. |
| `host` | `string` | yes | — | Inventory host running the service. |
| `network` | `string` | yes | — | Logical network containing the service. |
| `enabled` | `bool` | no | `true` | Whether the service participates in planning. |
| `address` | table | no | none | Container/logical service address. |
| `ports` | array<table> | no | `[]` | Host-to-container port mappings. Duplicate host/protocol pairs on one host are rejected. |
| `runtime` | `map<string>` | no | none | Provider-neutral runtime key/value metadata exposed to relevant planners. |
| `ingress` | table | no | none | Host ingress behavior. |
| `ssh` | table | no | none | SSH endpoint provided by the service. |
| `web` | table | no | none | Web and reverse-proxy intent. |

### `[services.<service>.address]`

| Key | Type | Required | Description |
|---|---|---:|---|
| `ipv4` | IPv4 | no | Must lie inside the service network's IPv4 CIDR and be globally unique among services. |
| `ipv6_interface_id` | `string` | no | Interface identifier combined with the network's resolved ULA subnet. |

### `[[services.<service>.ports]]`

| Key | Type | Required | Values | Description |
|---|---|---:|---|---|
| `host` | `u16` | yes | `1..65535` | Host-side port. |
| `container` | `u16` | yes | `1..65535` | Service/container-side port. |
| `protocol` | enum | yes | `tcp`, `udp` | Transport protocol. |

### `[services.<service>.ingress]`

| Key | Type | Required | Values | Description |
|---|---|---:|---|---|
| `mode` | enum | yes | `host-port` | How traffic enters the host. |
| `interface` | `string` | yes | — | Location interface used for ingress address resolution. |

### `[services.<service>.ssh]`

| Key | Type | Required | Description |
|---|---|---:|---|
| `user` | `string` | yes | SSH username exposed by the service. |
| `host_port` | `u16` | yes | Port exposed on the service host. Conflicts with normal service ports are rejected. |
| `container_port` | `u16` | yes | Internal SSH port. |
| `route` | `string` | yes | Route selector used by the SSH plan. |

### `[services.<service>.web]`

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `container_port` | `u16` | yes | — | Application port used as the reverse-proxy upstream. |
| `domain` | `string` | yes | — | Public or private hostname for the service. |
| `access` | `string` | yes | — | Access policy. Current reverse-proxy workflows use `reverse-proxy`. |
| `proxy` | `string` | yes | — | Service identifier of the reverse proxy. Must exist. |
| `scheme` | enum | no | `http` | Upstream scheme: `http` or `https`. |
| `tls` | `bool` | no | `false` | Enables TLS on the external listener. |
| `certificate` | `string` | no | none | Certificate reference from `[certificates]`. Recommended whenever `tls = true`. |
| `force_https` | `bool` | no | `false` | Renders HTTP-to-HTTPS redirect behavior where supported. |
| `websocket` | `bool` | no | `false` | Enables WebSocket proxy headers where supported. |
