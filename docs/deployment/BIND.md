# Deploy BIND

Current Nexus mount:

```text
/home/suhail/.local/share/netweft/generated/shane-xfinity/bind → /etc/bind
```

## Render

```bash
cd "$MATRIX/crates/netweft"
cargo run -- validate
cargo run -- show dns
cargo run -- render bind
```

## Stage

```bash
ssh suhail@10.214.90.10 '''
rm -rf /tmp/netweft-bind-staged
mkdir -p /tmp/netweft-bind-staged
'''
```

```bash
scp -r   "$HOME/.local/share/netweft/generated/shane-xfinity/bind/."   suhail@10.214.90.10:/tmp/netweft-bind-staged/
```

## Validate staged files

```bash
ssh -t suhail@10.214.90.10 '''
set -e
sudo docker run --rm   --entrypoint named-checkconf   -v /tmp/netweft-bind-staged:/etc/bind:ro   internetsystemsconsortium/bind9:9.20   /etc/bind/named.conf
'''
```

## Back up and install

```bash
ssh suhail@10.214.90.10 '''
set -e
backup="$HOME/.local/share/netweft/generated/shane-xfinity/bind.before-deploy-$(date +%Y%m%d-%H%M%S)"
cp -a "$HOME/.local/share/netweft/generated/shane-xfinity/bind" "$backup"
find "$HOME/.local/share/netweft/generated/shane-xfinity/bind"   -mindepth 1 -maxdepth 1 ! -name "*.jnl" -exec rm -rf {} +
cp -a /tmp/netweft-bind-staged/.   "$HOME/.local/share/netweft/generated/shane-xfinity/bind/"
echo "$backup"
'''
```

Do not delete the mounted parent directory itself.

## Validate and reload

RNDC is not configured. Use SIGHUP.

```bash
ssh -t suhail@10.214.90.10 '''
set -e
sudo docker exec bind9 named-checkconf /etc/bind/named.conf
sudo docker kill --signal=HUP bind9
sleep 2
sudo docker logs --tail 50 bind9
'''
```

## Verify

```bash
dig @10.214.90.10 quasar.suhail.ink A
dig @10.214.90.10 quasar.suhail.ink AAAA
dig @10.214.90.10 dsm.suhail.ink A
dig @10.214.90.10 cloudflare.com A
```

## Clean staging

```bash
ssh suhail@10.214.90.10 '''rm -rf /tmp/netweft-bind-staged'''
```
