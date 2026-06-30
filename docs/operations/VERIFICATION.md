# Verification

## Core

```bash
netweft validate
netweft adapters list
```

## Inspect plans

```bash
netweft show dns
netweft show proxy
netweft show docker --host nexus
netweft show host-network --host zion
netweft show os-network --host quasar
netweft show guests
netweft show proxmox-sdn --host zion
netweft show proxmox-storage --host zion
netweft show network-mounts --host vortex
netweft show nas-permissions
```

## Adapter verification

Use generated `verify.sh` where present:

```text
docker/verify.sh
proxmox-sdn/verify.sh
```

Other adapters use implementation-native checks documented in their deployment guide, such as `named-checkconf`, `nginx -t`, `netplan generate`, `pvesm status`, `findmnt`, and direct service tests.


## Cloudflare ingress

```bash
netweft show cloudflare
```

On the connector host:

```bash
docker ps --filter name=cloudflared
```

Verify each configured hostname with the generated `verify.sh`.
