# Rollback

BIND backups use:

```text
~/.local/share/netweft/generated/shane-xfinity/bind.before-deploy-<timestamp>
```

Restore files into the existing mounted directory, validate with `named-checkconf`, then send SIGHUP.

Native Nginx rollback remains documented in Dockyard:

```text
nexus/nginx-native/ROLLBACK.md
```
