# Verification

The patch was prepared from the uploaded `feature/nginx-adapter` snapshot at
`90ceddee7c55ec3a823dc9e28eff7dd4b16f3640`.

The artifact-building environment did not contain `cargo` or `rustfmt`, so the
Rust verification suite could not be executed here. Run the following after
applying the series:

```bash
cargo fmt --all
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
git diff --check
```

The source was checked for balanced Rust delimiters, NUL bytes, deterministic
fixture ordering, and a clean textual diff. No deployment files, credentials,
certificate material, or running-service state are included.
