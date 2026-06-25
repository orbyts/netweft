# Apply the native Nginx adapter patch series

Apply from a clean `feature/nginx-adapter` branch based on commit
`90ceddee7c55ec3a823dc9e28eff7dd4b16f3640`:

```bash
git am patches/*.patch
```

Verify:

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
git diff --check
```

Inspect without deployment:

```bash
cargo run -- adapters list
cargo run -- --config-dir tests/fixtures/proxy-services/config render nginx --host nexus
```

No container or running service is changed by this patch series.
