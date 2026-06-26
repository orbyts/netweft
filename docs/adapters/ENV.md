# Environment adapter

```bash
netweft validate
netweft show env --host nexus
netweft render env --host nexus
```

Output:

```text
~/.local/share/netweft/generated/<location>/hosts/<host>/
```

Typical files include `compose.env`, `shell.sh`, `shell.fish`, `shell.ps1`, and `manifest.txt`.

Generated files contain topology, not secrets.

See [environment deployment](../deployment/ENV.md).
