# bench-server

A small Rust HTTP server for load testing and RPS comparison across different frameworks/tools.

## Local Run

```bash
cargo run -- --port 8080
```

Dashboard: `http://127.0.0.1:8080/`

## Docker

Build:

```bash
docker build -t bench-server:local .
```

Run:

```bash
docker run --rm -p 8080:8080 bench-server:local
```

## Main Endpoints

- `GET /ok` -> `200`
- `GET /e404` -> `404`
- `GET /e500` -> `500`
- `GET /blob?min=256&max=65536`
- `GET /slow?chunks=5&min_chunk=256&max_chunk=4096`
- `GET /api/stats/summary`
- `GET /api/stats/timeseries`
- `POST /api/stats/reset`

## CI/CD Docker Image

Workflow: `.github/workflows/docker-image.yml`

- On `push` to `main` and on `v*` tags, it builds a multi-arch image (`linux/amd64`, `linux/arm64`) and pushes it to GHCR.
- On `pull_request`, it only builds the image (no push).
- Image path: `ghcr.io/<owner>/<repo>`

## Benchmark Contract

See [docs/benchmark-contract.md](docs/benchmark-contract.md).
