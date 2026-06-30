# Troubleshooting

## Sudo requires a terminal

```bash
ssh -t user@host '''sudo ...'''
```

## RNDC key missing

Use:

```bash
sudo docker kill --signal=HUP bind9
```

## Container sees old bind-mounted content

Do not replace the mounted parent directory. Replace files inside it.

## Git becomes dirty across Dropbox machines

Use independent Git clones and synchronize through GitHub.


## Cloudflare apply reports a missing token variable

The configured environment-variable names are printed by `netweft show cloudflare`. Export or source the corresponding values in the same shell that runs `apply-cloudflare.sh`. An `export` in one terminal process does not persist into a separate terminal or command execution.

## Cloudflare connector rollback did not restore DNS

This is expected. The generated `rollback.sh` restores only local connector deployment files. Tunnel configuration and public DNS reconciliation are remote state and are not automatically reversed.
