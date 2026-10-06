# Stackr Standard

USSD stablecoin settlement backend on Stellar.

## Commands

```bash
cargo run                      # Starts on :3000 (auto-runs migrations)
./scripts/test_ussd.sh         # 41 end-to-end USSD flow tests
cargo test                     # unit tests
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

## Architecture

```
User dials *384# → Telco → Africa's Talking → POST /ussd/callback
  → USSD state machine (Redis session) → Stellar network + Postgres + External APIs
```

## Source Layout

- `src/config/` — Environment config with validation
- `src/db/` — Postgres pool, migrations, models
- `src/middleware/` — Rate limiting, IP allowlist, security headers
- `src/routes/` — USSD callback, anchor webhooks, health check
- `src/services/ussd_menu/` — State machine (one file per USSD flow)
- `src/services/stellar/` — TX builder, wallet, transfers, swaps, anchors
- `src/services/anchor_orders.rs` — Anchor order lifecycle
- `src/services/price_feed.rs` — CoinGecko rates with Redis cache
- `src/services/bills/` — Airtime and utility bill payments
- `migrations/` — SQL migrations (applied on startup)
- `scripts/test_ussd.sh` — End-to-end test suite

## Key Design Decisions

- **Configurable fiat**: FIAT_CURRENCY / FIAT_COUNTRY env vars (default: RWF/RW)
- **Custodial wallets**: HKDF-SHA256 from master seed + phone
- **SEP-24 anchors**: Any compliant anchor works via config change
- **Fire-and-forget**: Activity + SMS via tokio::spawn, never blocks USSD
- **Idempotency**: Redis-backed keys on all money movements
- **Atomic order states**: Once completed/error/expired, immutable
