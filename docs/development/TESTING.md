# Testing

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
git diff --check
```

Important fixtures:

```text
tests/fixtures/shane-xfinity
tests/fixtures/proxy-services
```
