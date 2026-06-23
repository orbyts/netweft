# Netweft 0.2 internal refactor

This patch series changes architecture without intentionally changing existing
rendered behavior.

## Compatibility contract

These commands must continue to work:

```bash
netweft validate
netweft show config
netweft show hosts
netweft show networks
netweft show services
netweft show dns-access
netweft show dns
netweft show env --host nexus
netweft render bind
netweft render env --host nexus
netweft render all --host nexus
```

The current `shane-xfinity` configuration is captured under
`tests/fixtures/shane-xfinity/config` and exercised by compatibility tests.

## New internal flow

```text
configuration
    ↓
validation and provider-neutral resolution
    ↓
ResolvedPlan
    ↓
Adapter registry
    ├── BIND adapter
    └── environment adapter
    ↓
artifact manifest with target host
```

The new observation interface is intentionally empty by default:

```text
declared intent + ObservationSet::empty() → same result as Netweft 0.1
```

## Deliberately deferred

This series does not yet:

- split Netweft into separately published Cargo workspace crates;
- add Nginx or CoreDNS adapters;
- discover interfaces or devices;
- deploy files over SSH;
- reconcile observed state;
- change the TOML schema.

Keeping the first refactor inside one package reduces risk. Once the public
boundaries have compiled and stabilized, modules can be extracted into separate
crates without changing their responsibilities.
