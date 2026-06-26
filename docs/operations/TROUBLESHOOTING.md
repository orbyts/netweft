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
