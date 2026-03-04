# bench-server

Небольшой HTTP-сервер на Rust для нагрузочных тестов и сравнения RPS разных фреймворков/инструментов.

## Локальный запуск

```bash
cargo run -- --port 8080
```

Dashboard: `http://127.0.0.1:8080/`

## Docker

Сборка:

```bash
docker build -t bench-server:local .
```

Запуск:

```bash
docker run --rm -p 8080:8080 bench-server:local
```

## Основные эндпоинты

- `GET /ok` -> `200`
- `GET /e404` -> `404`
- `GET /e500` -> `500`
- `GET /blob?min=256&max=65536`
- `GET /slow?chunks=5&min_chunk=256&max_chunk=4096`
- `GET /api/stats/summary`
- `GET /api/stats/timeseries`
- `POST /api/stats/reset`

## CI/CD Docker image

Workflow: `.github/workflows/docker-image.yml`

- На `push` в `main` и на теги `v*` собирает multi-arch image (`linux/amd64`, `linux/arm64`) и пушит в GHCR.
- На `pull_request` только собирает image (без push).
- Путь образа: `ghcr.io/<owner>/<repo>`

## Контракт бенчмарка

См. [docs/benchmark-contract.md](docs/benchmark-contract.md).
