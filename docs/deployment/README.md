# Deployment

Netweft renders locally. Deployment is explicit and adapter-specific.

## First rule: inspect the rendered tree

Do not assume that every adapter emits the same files.

```bash
find ~/.local/share/netweft/generated -type f | sort
```

In particular, the Proxmox host-network adapter emits a filesystem payload only:

```text
hosts/<host>/proxmox/
├── etc/hosts
├── etc/network/interfaces
├── etc/resolv.conf
└── manifest.txt
```

It does **not** emit `action-plan.txt`, `install.sh`, `verify.sh`, or `rollback.sh`.

By contrast, adapters such as Docker, SSH, and Cloudflare emit guarded scripts. Consult [the adapter output table](../adapters/README.md) before writing deployment commands.

## General flow

```text
validate
→ inspect resolved plan
→ render
→ inspect generated tree
→ back up target state
→ stage artifacts
→ run provider-native validation
→ apply explicitly
→ verify
→ retain rollback path
```

## Safety classes

### Filesystem payload or manual-plan adapters

- BIND;
- environment files;
- Nginx;
- Proxmox host networking;
- Synology NFS permission plans.

These require a deployment procedure documented for that provider. Netweft does not invent a generic installer for them.

### Guarded script adapters

- Cloudflare ingress;
- Docker networking;
- Netplan;
- Proxmox guests;
- Proxmox SDN;
- Proxmox storage;
- SSH client configuration;
- systemd network mounts.

Inspect every generated script before execution.

## Remote network changes

A host-network change can terminate SSH. Keep an out-of-band console available, back up the current files, and understand the cluster implications before activation. For Proxmox clusters, preserve quorum and migrate Corosync addresses deliberately.

Use [Cross-host transfer](CROSS-HOST-TRANSFER.md) when rendering on another machine.
