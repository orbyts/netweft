# Deploy Cloudflare ingress

Cloudflare deployment has two distinct trust zones:

1. a trusted client with Cloudflare API-token environment variables;
2. the connector host that runs `cloudflared`.

The API reconciliation script should run only on the trusted client. The connector host receives only the generated tunnel connector token.

## 1. Inspect and render

```bash
cd "$MATRIX/crates/netweft"

cargo run -- validate
cargo run -- show cloudflare
cargo run -- render cloudflare --tunnel nexus-ingress
```

Inspect all generated files before applying:

```bash
find \
  "$HOME/.local/share/netweft/generated/shane-xfinity/hosts/nexus/cloudflare/nexus-ingress" \
  -maxdepth 2 \
  -type f \
  -print | sort

sed -n '1,240p' \
  "$HOME/.local/share/netweft/generated/shane-xfinity/hosts/nexus/cloudflare/nexus-ingress/action-plan.txt"
```

## 2. Reconcile the remote tunnel and DNS

Make both configured API-token variables available in the current trusted shell. Then run:

```bash
cd \
  "$HOME/.local/share/netweft/generated/shane-xfinity/hosts/nexus/cloudflare/nexus-ingress"

./apply-cloudflare.sh
```

The script:

- creates the named tunnel if absent, otherwise adopts it;
- replaces the tunnel ingress configuration;
- creates or updates each proxied CNAME;
- obtains the connector token;
- writes `deployment.env` with mode constrained by `umask 077`.

Confirm the secret file exists without printing it:

```bash
test -s deployment.env
stat -f '%Sp %N' deployment.env 2>/dev/null || stat -c '%A %n' deployment.env
```

Do not commit, paste, or log `deployment.env`.

## 3. Stage on Nexus

```bash
ssh suhail@10.214.90.10 '
rm -rf /tmp/netweft-cloudflare-staged
mkdir -p /tmp/netweft-cloudflare-staged
'
```

```bash
scp -r \
  "$HOME/.local/share/netweft/generated/shane-xfinity/hosts/nexus/cloudflare/nexus-ingress/." \
  suhail@10.214.90.10:/tmp/netweft-cloudflare-staged/
```

Because the staged directory contains `deployment.env`, remove it after installation.

## 4. Install the connector

```bash
ssh -t suhail@10.214.90.10 '
set -e
cd /tmp/netweft-cloudflare-staged
sudo ./install.sh
'
```

The installer writes:

```text
/var/lib/netweft/cloudflare/compose.yml
/var/lib/netweft/cloudflare/.env
```

and starts the `cloudflared` container with host networking. Existing connector files are backed up under:

```text
/root/netweft-backups/cloudflare-YYYYMMDD-HHMMSS
```

## 5. Verify

```bash
ssh -t suhail@10.214.90.10 '
cd /tmp/netweft-cloudflare-staged
sudo ./verify.sh
'
```

The generated verification script checks the container and requests every configured HTTPS hostname. It currently uses `curl -k`; therefore it verifies reachability and HTTP response, not origin certificate trust.

## 6. Remove staging

```bash
ssh suhail@10.214.90.10 '
rm -rf /tmp/netweft-cloudflare-staged
'
```

## Rollback boundary

Connector files can be restored with the generated script:

```bash
ssh -t suhail@10.214.90.10 '
cd /path/to/rendered-or-restaged/cloudflare-artifacts
sudo ./rollback.sh /root/netweft-backups/cloudflare-TIMESTAMP
'
```

This rollback only restores the connector Compose and environment files. It does **not** revert Cloudflare tunnel configuration or DNS records. Remote Cloudflare changes require a separate deliberate reconciliation or manual API/dashboard rollback.
