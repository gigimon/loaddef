# Step 1 Foundation

## Цель
Собрать рабочий benchmark-сервер и зафиксировать контракт для честного сравнения нагрузочных инструментов.

## Реализация
- [x] Инициализирован Rust-проект `bench-server`.
- [x] Добавлен каркас `axum + tokio`, конфиг через CLI/env, graceful shutdown, `/healthz`.
- [x] Реализованы endpoint: `/ok`, `/e404`, `/e500`, `/blob`, `/slow`.
- [x] Реализован сбор статистики в памяти: global counters, by endpoint, by status, latency histograms, second-level timeseries.
- [x] Реализованы API статистики: `/api/stats/summary`, `/api/stats/timeseries`, `/api/stats/reset`.
- [x] Реализован встроенный dashboard `GET /` (polling 1s, график RPS, summary, breakdown).
- [x] Зафиксирован контракт и протокол сравнения в `docs/benchmark-contract.md`.

## Проверка
- [x] `cargo fmt`
- [x] `cargo check`
- [x] Smoke через `curl` по ключевым endpoint и API статистики.

## Ограничения текущей итерации
- [ ] `GET /metrics` в формате Prometheus пока не реализован (опциональный пункт Step 4).
- [ ] Автоматизация профилей и интеграция с внешними нагрузочными инструментами в следующих шагах.
