# Deploy OpenSSH client aliases

Render for the client that will consume the aliases:

```bash
netweft validate
netweft show ssh --client quasar
netweft render ssh --client quasar
```

If rendered on that same client:

```bash
cd "$HOME/.local/share/netweft/generated/shane-xfinity/clients/quasar/ssh"
./install.sh
./verify.sh
```

The installer preflights every generated alias with `ssh -G`, atomically replaces only `~/.ssh/config.d/netweft`, and stores the previous directory under:

```text
~/.local/state/netweft/ssh-backups/01234567-012345
```

Rollback:

```bash
./rollback.sh \
  "$HOME/.local/state/netweft/ssh-backups/TIMESTAMP"
```

The root `~/.ssh/config` remains outside Netweft ownership. It must include the generated snippet directory, for example:

```sshconfig
Include ~/.ssh/config.d/netweft/*.conf
```
