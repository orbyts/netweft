# `netweft.toml`

Controls global paths, active location selection, rendering behavior, and warning policy.

## Complete example

```toml
schema_version = 1
active_location = "shane-xfinity"

[paths]
# generated_root = "/custom/generated"
# state_root = "/custom/state"
# cache_root = "/custom/cache"

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

## Top-level keys

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `schema_version` | `u32` | yes | — | Configuration schema version. Current supported value is `1`. |
| `active_location` | `string` | yes | — | Basename of the location file under `locations/`, without `.toml`. May be overridden by `--location`. |
| `paths` | table | no | empty | Optional path overrides. |
| `render` | table | no | all defaults | Rendering behavior. |
| `validation` | table | no | all `false` | Warning behavior. |

## `[paths]`

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `generated_root` | `string` | no | XDG data path | Root used for generated artifacts. |
| `state_root` | `string` | no | XDG state path | Persistent Netweft state root. |
| `cache_root` | `string` | no | XDG cache path | Rebuildable cache root. |

Use `netweft paths` to see resolved paths.

## `[render]`

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `atomic` | `bool` | no | `true` | Requests atomic output replacement where the renderer supports it. |
| `stable_order` | `bool` | no | `true` | Requests deterministic ordering in generated files. |
| `generated_headers` | `bool` | no | `true` | Adds generated-file warnings or provenance headers where supported. |

## `[validation]`

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `warn_dynamic_ipv6` | `bool` | no | `false` | Enables warnings when dynamic router-advertised IPv6 is present and cannot be treated as durable allocation space. |
| `warn_dropbox_runtime` | `bool` | no | `false` | Enables warnings for runtime paths placed under Dropbox-like synchronized storage. |
| `warn_latest_images` | `bool` | no | `false` | Enables warnings for unpinned container images using tags such as `latest`. |
| `deny_warnings` | `bool` | no | `false` | Converts validation warnings into errors. |

## Location override

```bash
netweft validate --location la-unifi
```

The override changes the loaded location for that invocation; it does not edit `active_location`.
