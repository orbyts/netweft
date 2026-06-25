# Native Nginx adapter

The official `nginx` adapter consumes the provider-neutral proxy plan. It does
not read service TOML directly and it does not issue or renew certificates.

## Render

```bash
netweft render nginx
netweft render nginx --host nexus
```

When all resolved proxy entries target one host, `--host` is optional. If a
plan targets multiple proxy hosts, the host must be selected explicitly.

The generated tree is:

```text
generated/<location>/hosts/<host>/nginx/
├── nginx.conf
├── conf.d/
│   └── <domain>.conf
└── manifest.txt
```

Rendering is local and does not restart or modify a running Nginx process.
`render all` retains its existing BIND and environment behavior; native Nginx
rendering remains explicit during this migration phase.

## Validate with Nginx

```bash
netweft render nginx --host nexus --check
netweft render nginx --host nexus --check --nginx /path/to/nginx
```

The check runs `nginx -t` against the rendered root. Certificate paths are
opaque runtime mount paths, so the check environment must expose those files
at the declared paths. A deployment container can perform the same check after
mounting generated configuration and certificates read-only.

## Certificate boundary

The adapter requires a resolved certificate reference for every TLS listener.
It renders only `ssl_certificate` and `ssl_certificate_key` paths. ACME
accounts, DNS credentials, issuance, renewal, and reload orchestration remain
outside Netweft.

## Parallel deployment

Nginx Proxy Manager should remain on ports 80, 81, and 443 while native Nginx
is tested separately. A later deployment phase may map native Nginx to host
ports 8080 and 8443, validate every hostname and WebSocket route, and only then
consider a controlled cutover.
