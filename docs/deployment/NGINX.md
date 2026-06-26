# Deploy native Nginx

## Render

```bash
cd "$MATRIX/crates/netweft"
cargo run -- validate
cargo run -- show proxy
cargo run -- render nginx --host nexus
```

## Stage when rendered off Nexus

```bash
ssh suhail@10.214.90.10 '''
rm -rf /tmp/netweft-nginx-staged
mkdir -p /tmp/netweft-nginx-staged
'''
```

```bash
scp -r   "$HOME/.local/share/netweft/generated/shane-xfinity/hosts/nexus/nginx/."   suhail@10.214.90.10:/tmp/netweft-nginx-staged/
```

Install into Nexus's generated tree, then run:

```bash
ssh -t suhail@10.214.90.10 '''
set -e
"$HOME/Dropbox/matrix/docker/services/nexus/nginx-native/deploy-generated-config.sh"
'''
```

The Dockyard script copies files into the runtime bind mount, runs `nginx -t`, and reloads Nginx.
