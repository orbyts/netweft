# Deploy Proxmox host networking

This adapter renders owned host-network files but does not render an apply script. Host networking is high risk and should be activated from a Proxmox console or another out-of-band path.

## Render

```bash
netweft validate
netweft show host-network --host zion
netweft render proxmox --host zion
```

## Output ownership

```text
etc/network/interfaces
etc/hosts
etc/resolv.conf
```

The manifest explicitly preserves:

```text
/etc/network/interfaces.d/*
```

and excludes SDN and corosync state.

## Recommended activation

1. Copy the rendered directory to the target host.
2. Diff every generated file against the live file.
3. Back up the live files.
4. Install only the owned files.
5. Validate ifupdown2 syntax.
6. Apply with `ifreload -a` from console access.
7. Verify management connectivity, routes, DNS, and Proxmox cluster health.

Do not perform the first deployment through the only active SSH path.
