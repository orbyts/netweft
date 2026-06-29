# Synology NFS permission adapter

Renders a location-resolved DSM action plan for NFS share permissions. It intentionally does not automate DSM changes.

## Inspect and render

```bash
netweft show nas-permissions --nas ds1621plus
netweft render synology-nfs-permissions --nas ds1621plus
```

## Output

```text
~/.local/share/netweft/generated/<location>/nas/<nas>/nfs-permissions/
├── action-plan.txt
└── manifest.txt
```

Rendering does not apply the artifact. See the matching deployment guide.
