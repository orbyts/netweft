# Netweft adapter SDK

Netweft adapters translate one validated, resolved network plan into files for a
specific implementation such as BIND, Nginx, CoreDNS, Traefik, nftables, or a
host shell environment.

The adapter boundary deliberately separates four concerns:

1. TOML configuration describes declared intent.
2. observation providers may report current state without modifying it;
3. `ResolvedPlan` provides provider-neutral derived data;
4. adapters render tool-specific artifacts and return a deployment manifest.

Adapters do not deploy files in this version.

## Built-in adapters

```text
bind  authoritative and recursive BIND 9 configuration
env   Docker Compose and shell environment files for one host
```

List adapters compiled into the CLI:

```bash
netweft adapters list
```

Existing commands remain compatible:

```bash
netweft render bind
netweft render env --host nexus
netweft render all --host nexus
```

## Implementing an adapter

A Rust adapter implements `netweft::adapter::Adapter`:

```rust
use anyhow::Result;
use netweft::adapter::{
    Adapter, AdapterContext, AdapterId, AdapterMetadata, AdapterOutput,
    Capability,
};

pub struct ExampleAdapter;

impl Adapter for ExampleAdapter {
    fn metadata(&self) -> AdapterMetadata {
        AdapterMetadata {
            id: AdapterId::new("example"),
            name: "Example",
            description: "Render an example network configuration",
            capabilities: &[Capability::ReverseProxy],
        }
    }

    fn validate(&self, context: &AdapterContext<'_, '_>) -> Result<()> {
        // Ask context.plan for provider-neutral resolved data.
        let _configuration = context.plan.config();
        Ok(())
    }

    fn render(&self, context: &AdapterContext<'_, '_>) -> Result<AdapterOutput> {
        // Write deterministic files beneath an adapter-owned output root,
        // collect an artifact manifest, and return its intended target host.
        todo!()
    }
}
```

Register it in a downstream application:

```rust
let mut registry = netweft::adapter::registry::AdapterRegistry::new();
registry.register(ExampleAdapter)?;
```

## Rules for adapters

Adapters should:

- consume `ResolvedPlan` rather than reparsing TOML;
- produce deterministic output;
- keep secrets outside generated artifacts;
- own a distinct output subtree;
- report the host that should receive the artifacts;
- avoid changing the network during `validate` or `render`;
- return errors rather than silently guessing.

An adapter may add provider-specific configuration, but generic network intent
should remain in Netweft's provider-neutral model.

## Observation boundary

`netweft::observe` defines read-only observation providers for future location,
interface, route, Docker, Tailscale, and device discovery.

No observer runs implicitly. This preserves today's deterministic behavior:
configuration remains the source of truth until the user explicitly requests an
observation or reconciliation operation.

## Future external adapters

Rust adapters are initially compiled into a Netweft binary. A later version may
support external processes using a versioned JSON protocol. That will allow
adapters written in Rust, Go, Python, or other languages without relying on an
unstable Rust dynamic-library ABI.
