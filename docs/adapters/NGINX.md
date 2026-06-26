# Native Nginx adapter

```bash
netweft validate
netweft show proxy
netweft render nginx --host nexus
```

Output:

```text
~/.local/share/netweft/generated/<location>/hosts/<host>/nginx/
├── nginx.conf
├── conf.d/
└── manifest.txt
```

The adapter does not issue certificates or reload Nginx.

See [Nginx deployment](../deployment/NGINX.md).
