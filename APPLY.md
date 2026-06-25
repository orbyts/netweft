# Applying the Netweft proxy-model patch series

## Preconditions

- Start from commit `0c74b58b520559c4cd38c8299b7199c553df493d` on `feature/proxy-model`.
- Keep the working tree clean, or commit/stash local work first.
- Review every patch before applying it.

## Apply patches

```bash
tar -xzf netweft-proxy-model-patch-series.tar.gz
cd netweft-proxy-model-patch-series
git am patches/*.patch
```

## Verify

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
git diff --check
```

Inspect the resolved plan with a configuration that declares proxy intent:

```bash
netweft validate
netweft show proxy
```

This phase does not render Nginx configuration, restart containers, issue certificates, or modify live deployment files.

## Fallback archive

The companion patched-source archive contains the complete source tree after the patch series, excluding `.git` and build output. It can be compared against an applied tree or used as a clean fallback copy.
