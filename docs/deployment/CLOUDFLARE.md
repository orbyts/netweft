# Deploy Cloudflare ingress

Cloudflare deployment may mutate remote tunnel and DNS state and emit a secret connector token. Review the plan, provide API tokens through the configured environment variables, render/reconcile explicitly, protect generated secret files, install the connector on `connector_host`, and verify both Cloudflare and origin health. Local rollback does not automatically revert remote DNS or tunnel state.
