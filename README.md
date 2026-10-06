# Stackr

USSD stablecoin payments on Stellar. Send money, pay bills, and cash out — from any phone, on any network. No app, no internet, no smartphone required.

## What is Stackr?

Stackr turns any mobile phone into a stablecoin wallet. Users dial a USSD code (*384#) to access financial services powered by the Stellar blockchain. The backend handles wallet creation, Stellar transactions, currency conversion, and settlement — all within USSD latency constraints.

**Live:** [usestackr.xyz](https://usestackr.xyz) &middot; **API:** [api.usestackr.xyz](https://api.usestackr.xyz) &middot; **Docs:** [docs.usestackr.xyz](https://docs.usestackr.xyz)

## How It Works

```
User dials *384# → Telco → Africa's Talking → POST /ussd/callback
  → USSD state machine (Redis session) → Stellar network + Postgres + External APIs
```

1. User dials a USSD code from any phone (2G, 3G, 4G — feature phone or smartphone)
2. Stackr processes the request through a state machine, handles Stellar transactions
3. Money arrives in ~5 seconds with SMS confirmation

## Features

- **Send Money** — Transfer USDC to any phone number
- **Pay Bills** — Airtime, electricity, cable TV, internet via USDC
- **Cash In / Cash Out** — Deposit and withdraw via SEP-24 anchors (mobile money, bank)
- **Merchant Payments** — Pay merchants by code, instant USDC settlement
- **Token Swaps** — USDC/XLM swaps on the Stellar DEX
- **Auto Wallets** — Every phone number gets a Stellar wallet automatically

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Backend | Rust (Actix-web) |
| Database | PostgreSQL (Neon) |
| Cache/Sessions | Redis (Upstash) |
| Blockchain | Stellar (USDC) |
| USSD Gateway | Africa's Talking |
| Bill Payments | Airbills API |
| Website | Next.js 14 (static export) |
| Auth | Supabase |
| Hosting | VPS + Nginx + Let's Encrypt |

## API

17 REST endpoints. Full documentation at [docs.usestackr.xyz](https://docs.usestackr.xyz).

```bash
# Health check
curl https://api.usestackr.xyz/health

# Send USDC (requires API key)
curl -X POST https://api.usestackr.xyz/api/v1/transfers \
  -H "Authorization: Bearer sk_live_..." \
  -H "Content-Type: application/json" \
  -d '{"sender_phone":"+250781234567","recipient_phone":"+250789876543","amount":"10.00","pin":"1234"}'
```

### Endpoints

| Category | Endpoints |
|----------|-----------|
| Core | `GET /health`, `POST /ussd/callback`, `POST /webhooks/anchor` |
| Accounts | `POST /api/v1/accounts`, `GET /api/v1/accounts/{phone}`, `GET /api/v1/accounts/{phone}/balance` |
| Transfers | `POST /api/v1/transfers` |
| Rates | `GET /api/v1/rates` |
| Transactions | `GET /api/v1/transactions` |
| Deposits | `POST /api/v1/deposits`, `GET /api/v1/deposits/{id}` |
| Withdrawals | `POST /api/v1/withdrawals`, `GET /api/v1/withdrawals/{id}` |
| Payments | `POST /api/v1/payments/merchant`, `POST /api/v1/payments/airtime`, `POST /api/v1/payments/bill` |
| Swaps | `POST /api/v1/swaps` |

## Project Structure

```
src/
├── config/          # Environment config with validation
├── db/              # Postgres pool, migrations, models
├── middleware/       # Rate limiting, IP allowlist, security headers
├── routes/          # USSD callback, API v1 endpoints, webhooks
├── services/
│   ├── ussd_menu/   # State machine (one file per USSD flow)
│   ├── stellar/     # TX builder, wallet, transfers, swaps, anchors
│   ├── bills/       # Airtime and utility bill payments
│   ├── price_feed.rs
│   └── sms.rs
├── utils/           # Phone validation, crypto helpers
└── main.rs

website/             # Next.js 14 static site
├── src/app/         # Pages (docs, dashboard, auth)
├── src/components/  # UI components
└── src/lib/         # Supabase client, auth, USSD simulator data

migrations/          # SQL migrations (applied on startup)
deploy/              # Nginx config, systemd service, deploy scripts
```

## Getting Started

### Prerequisites

- Rust 1.75+
- PostgreSQL
- Redis
- Stellar testnet account

### Setup

```bash
# Clone
git clone https://github.com/Winni3ndy/STACKR.git
cd STACKR

# Configure
cp .env.example .env
# Edit .env with your credentials

# Run (auto-applies migrations)
cargo run

# Server starts on :3000
```

### Website

```bash
cd website
npm install
npm run dev
# Site runs on :3001
```

## Key Design Decisions

- **Configurable fiat** — `FIAT_CURRENCY` / `FIAT_COUNTRY` env vars (default: RWF/RW)
- **Custodial wallets** — HKDF-SHA256 from master seed + phone number
- **SEP-24 anchors** — Any compliant anchor works via config change
- **Fire-and-forget** — Activity logs + SMS via `tokio::spawn`, never blocks USSD
- **Idempotency** — Redis-backed keys on all money movements
- **Atomic order states** — Once completed/error/expired, immutable

## License

MIT
