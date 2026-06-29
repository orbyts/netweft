# Deploy Docker networking

This adapter may restart Docker, recreate named networks, stop and restart Compose projects, and rewrite references to renamed networks. Treat it as disruptive.

## Render

```bash
netweft validate
netweft show docker --host nexus
netweft render docker --host nexus
```

## Transfer

Copy the complete generated directory to the target host. Then inspect:

```bash
cd ~/.local/share/netweft/generated/<location>/hosts/<host>/docker
cat action-plan.txt
cat daemon.desired.json
sed -n '1,320p' apply.sh
```

## Apply

Run locally on the target host as root or through sudo:

```bash
sudo ./apply.sh
```

The script:

- backs up `/etc/docker/daemon.json`;
- merges desired daemon settings with the existing file using `jq`;
- validates with `dockerd --validate`;
- records running containers;
- restarts Docker;
- reconciles named networks;
- updates affected Compose files where required;
- restores projects and previously running containers.

## Verify

```bash
sudo ./verify.sh
```

## Roll back

```bash
sudo ./rollback.sh /root/netweft-backups/docker-HOST-TIMESTAMP
```

Schedule a maintenance window and preserve out-of-band access when changing a remote Docker host.
