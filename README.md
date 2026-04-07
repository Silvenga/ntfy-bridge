# ntfy-bridge

A simple bridge for [ntfy](https://ntfy.sh) that forwards webhooks from various services to ntfy topics.

## Features

- Supports Netdata alerts and reachability notifications.
- Bearer token authentication for API access.
- Built-in rate limiting.

## Environment Variables

The following environment variables can be used to configure the bridge.

| Variable                | Default           | Description                     |
|-------------------------|-------------------|---------------------------------|
| `NTFY_URL`              | `https://ntfy.sh` | ntfy server URL                 |
| `API_TOKEN`             |                   | API token for authentication    |
| `LISTEN_ADDR`           | `0.0.0.0:8080`    | Address to listen on            |
| `RATE_LIMIT_PER_SECOND` | `2`               | Rate limit requests per second  |
| `RATE_LIMIT_BURST`      | `5`               | Rate limit burst size           |
| `NTFY_TOKEN`            |                   | ntfy access token               |
| `NTFY_USERNAME`         |                   | ntfy username                   |
| `NTFY_PASSWORD`         |                   | ntfy password                   |
