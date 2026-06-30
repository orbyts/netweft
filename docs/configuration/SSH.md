# `ssh.toml`

Optional. Defines generated SSH client configuration for named clients.

```toml
schema_version = 1

[clients.quasar.identities.primary]
file = "~/.ssh/id_ed25519"

[targets.nexus]
host = "nexus"
interface = "lan"
user = "suhail"
port = 22
identity = "primary"
forward_agent = false
```

## `[clients.<client>.identities.<identity>]`

| Key | Type | Required | Description |
|---|---|---:|---|
| `file` | `string` | yes | Private-key path written to generated SSH configuration. Netweft does not copy or read the key. |

## `[targets.<target>]`

Exactly one practical endpoint selector should identify the target.

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `host` | `string` | conditional | none | Inventory host identifier. |
| `guest` | `string` | conditional | none | Guest identifier from `guests.toml`. |
| `service` | `string` | conditional | none | Service identifier exposing SSH. |
| `interface` | `string` | no | none | Location interface to use for a host target. |
| `user` | `string` | yes | SSH login user. |
| `port` | `u16` | no | `22` | SSH port. |
| `identity` | `string` | yes | Identity name from the selected client's identities. |
| `forward_agent` | `bool` | no | `false` | Writes agent-forwarding policy. Enable only when the trust model requires it. |

Each client is resolved independently:

```bash
netweft show ssh --client quasar
netweft render ssh --client quasar
```
