# Cloudflare ingress adapter

Netweft models external ingress per location. `cloudflare-tunnel` mode creates or adopts a remotely managed Cloudflare Tunnel, publishes application hostnames to `<tunnel-id>.cfargotunnel.com`, and runs `cloudflared` on the connector host. The zone apex is never changed by this adapter.

The API token value remains outside Netweft. Configure only its environment-variable name, such as `CLOUDFLARE_API_TOKEN_SUHAIL_INK`; Apogee may supply the value at shell initialization.

```sh
netweft show cloudflare
netweft render cloudflare --tunnel nexus-ingress
```

Run `apply-cloudflare.sh` on the trusted client where the API-token environment variable exists. It writes `deployment.env`, which contains the connector token and must be handled as a secret. Transfer the rendered directory to the connector host, then run `sudo ./install.sh` there.

Direct-origin mode is reserved in the schema but intentionally unsupported in this release.
