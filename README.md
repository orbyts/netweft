# Netweft

Deterministic network planning and configuration generation for portable infrastructure.

Netweft will model hosts, services, logical networks, and location-specific
profiles, then validate and render configuration for BIND, Nginx, Docker
Compose, SSH, Tailscale, and Apogee.

## Status

This initial `0.0.1` release reserves the package name while the first
implementation is developed.

## Usage

```console
$ netweft
netweft: network planning and configuration generation is coming soon
```

## Planned commands

```console
netweft validate --profile <profile>
netweft plan --profile <profile>
netweft render --profile <profile>
netweft diff --from <profile> --to <profile>
netweft ula generate
```

## License

Licensed under the MIT License.
