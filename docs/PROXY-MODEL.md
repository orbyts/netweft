# Provider-neutral proxy model

```text
service configuration → proxy intent → ResolvedProxyPlan → provider adapter
```

The model describes domains, listeners, upstreams, TLS policy, certificate references, forced HTTPS, WebSockets, and deployment target hosts.

```bash
netweft show proxy
```

Netweft validates certificate references but does not issue or renew certificates.
