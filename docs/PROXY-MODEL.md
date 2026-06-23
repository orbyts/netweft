# Provider-neutral proxy model

Netweft resolves reverse-proxy intent before any provider adapter renders files:

```text
service configuration -> proxy intent -> ResolvedProxyPlan -> provider adapters
```

The core model describes listeners, domains, upstreams, TLS policy, certificate references, and WebSocket requirements. It does not contain Nginx, Traefik, Caddy, or HAProxy directives.

## Service intent

A web service keeps its application endpoint and ingress intent together:

```toml
[services.jellyfin.web]
container_port = 8096
domain = "jellyfin.suhail.ink"
access = "reverse-proxy"
proxy = "nginx"
scheme = "http"
tls = true
certificate = "wildcard-suhail"
force_https = true
websocket = true
```

`proxy` names the reverse-proxy service. The resolver uses that service's host as the deployment target while preserving the web service's address and port as the upstream. This allows a proxy on one host to target either a container network address or a physical host address.

## Certificate boundary

Certificate declarations are references to files mounted into a future proxy runtime:

```toml
[certificates.wildcard-suhail]
domains = ["suhail.ink", "*.suhail.ink"]
certificate_path = "/etc/netweft/certificates/wildcard-suhail/fullchain.pem"
private_key_path = "/etc/netweft/certificates/wildcard-suhail/privkey.pem"
```

Netweft validates identifiers, absolute paths, domain coverage, and references. It does not read private-key contents, issue certificates, renew certificates, store ACME credentials, or manage DNS-provider tokens. Certbot, `acme.sh`, or another certificate manager remains responsible for those operations.

For compatibility, an existing `tls = true` declaration without `certificate` still resolves. New managed configurations should use an explicit certificate reference before a provider adapter is deployed.

## Inspection

```bash
netweft show proxy
```

This prints the deterministic provider-neutral plan and does not render files or modify a running service.
