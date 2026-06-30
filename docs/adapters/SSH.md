# OpenSSH client adapter

The `ssh` adapter resolves location-aware SSH aliases for one client profile. Targets may refer to physical hosts, guests, or SSH-enabled services.

## Inspect and render

```bash
netweft show ssh --client quasar
netweft render ssh --client quasar
```

## Output

```text
~/.local/share/netweft/generated/<location>/clients/<client>/ssh/
├── action-plan.txt
├── config.d/netweft/<alias>.conf
├── install.sh
├── verify.sh
└── rollback.sh
```

The installer owns only `~/.ssh/config.d/netweft`. It expects the user-maintained `~/.ssh/config` include chain to include that directory. Identity files are referenced, never copied or generated.

Deployment: [OpenSSH client deployment](../deployment/SSH.md)
