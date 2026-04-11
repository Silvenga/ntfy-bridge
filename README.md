# ntfy-bridge

A simple bridge for [ntfy](https://ntfy.sh) that forwards webhooks from various services to Ntfy topics.

## Features

- Supports Netdata alerts and reachability notifications.
- Bearer token authentication for API access.
- Built-in rate limiting.

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
