# ntfy-bridge

A simple bridge for [ntfy](https://ntfy.sh) that forwards webhooks from various services to Ntfy topics. Written in
Rust.

## Supported Services

### Netdata

```http
POST /api/v1/<ntfy topic>/netdata
```

Alert, reachability, and token webhooks from Netdata are supported.

## Configuration

The bridge can be configured via command-line options, environment variables, or both.

```text
Usage: ntfy-bridge [OPTIONS]

Options:
      --ntfy-url <NTFY_URL>            Ntfy server URL [env: NTFY_URL=https://ntfy.sh] [default: https://ntfy.sh]
      --ntfy-username <NTFY_USERNAME>  Ntfy username [env: NTFY_USERNAME=]
      --ntfy-password <NTFY_PASSWORD>  Ntfy password [env: NTFY_PASSWORD=]
      --ntfy-token <NTFY_TOKEN>        Ntfy access token [env: NTFY_TOKEN=]
      --api-token <API_TOKEN>          API token for authentication [env: API_TOKEN=]
      --listen-addr <LISTEN_ADDR>      Address to listen on [env: LISTEN_ADDR=] [default: 0.0.0.0:8080]
      --base-path <BASE_PATH>          Base path for the API [env: BASE_PATH=] [default: api]
      --log-level <LOG_LEVEL>          Log level [env: LOG_LEVEL=] [default: info]
  -h, --help                           Print help
  -V, --version                        Print version
```

If `--api-token` (`API_TOKEN`) is configured, the bridge will require clients to include an `Authorization` header with
the value `Bearer <API token>`.

The base path for the API can be configured via `--base-path` (`BASE_PATH`). For example, if the base path is set to
`webhooks`, the Netdata endpoint would be `/webhooks/v1/<ntfy topic>/netdata`. This defaults to `/api/v1`. This is
useful if you want to host the ntfy-bridge on the same host as ntfy.

## Usage

```bash
docker run ghcr.io/silvenga/ntfy-bridge:latest
```

Ntfy has a health endpoint at `/api/v1/health`.
