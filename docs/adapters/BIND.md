# BIND adapter

```bash
netweft validate
netweft show dns
netweft render bind
```

Output:

```text
~/.local/share/netweft/generated/<location>/bind/
```

The adapter renders authoritative zones, reverse zones, recursion ACLs, forwarding configuration, and a manifest. It does not deploy or reload BIND.

See [BIND deployment](../deployment/BIND.md).
