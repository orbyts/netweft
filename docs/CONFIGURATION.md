# Configuration

```text
~/.config/netweft/
├── netweft.toml
├── inventory.toml
├── networks.toml
├── services.toml
├── dns.toml
├── allocations.toml
└── locations/
```

| File | Responsibility |
|---|---|
| `netweft.toml` | Global settings and active location |
| `inventory.toml` | Stable host identities |
| `networks.toml` | Stable logical networks |
| `services.toml` | Services and placement |
| `dns.toml` | Zones, records, recursion, forwarding |
| `allocations.toml` | Durable ULA allocation IDs |
| `locations/*.toml` | Site-specific addressing and routing |

Dynamic ISP prefixes are not durable host identity. Secrets remain outside configuration.
