# Stackr Standard — Documentation

## Table of Contents

- [Architecture](#architecture)
- [USSD Protocol](#ussd-protocol)
- [Wallet System](#wallet-system)
- [Stellar Integration](#stellar-integration)
- [Anchor Integration (SEP-10 / SEP-24)](#anchor-integration)
- [Price Feed](#price-feed)
- [Bill Payments](#bill-payments)
- [Database Schema](#database-schema)
- [API Reference](#api-reference)
- [Security](#security)
- [Production Deployment](#production-deployment)
- [Troubleshooting](#troubleshooting)

---

## Architecture

```
User dials *384# on any phone
        │
  Telco USSD Gateway
        │
  Africa's Talking (POST /ussd/callback)
        │
  ┌─────▼──────┐
  │  actix-web  │──── IP allowlist + rate limiting (Redis)
  └─────┬──────┘
        │
  USSD State Machine (Redis session + Postgres user lookup)
        │
  ┌─────┼──────────────────┐
  │     │                  │
Stellar     Postgres        External APIs
─────────   ────────        ─────────────
TX builder  users           SEP-24 anchors
transfers   transactions    CoinGecko (rates)
swaps       anchor_orders   Airbills (bills)
trustlines  user_activity   Africa's Talking (SMS)
fee bumps   merchants
```

### Design Principles

1. **USSD latency is sacred.** USSD sessions timeout after ~30 seconds. Every handler must respond fast. Heavy operations (activity logging, SMS) are fire-and-forget via `tokio::spawn`.

2. **Config over code.** Deploying to a new country should require changing env vars, not source code. Currency, anchor, and shortcode are all configurable.

3. **Atomic state transitions.** Anchor orders and financial operations use atomic database transitions. Once an order reaches a terminal state (completed/error/expired), it cannot be changed.

4. **Fail safe.** If a bill payment API fails after debiting USDC, the system auto-refunds. If the refund fails, it logs for manual resolution. Money is never silently lost.

5. **No apps, no internet.** The entire system is accessible via USSD. Users need only a phone number and a 4-digit PIN.

---

## USSD Protocol

Stackr uses the [Africa's Talking USSD API](https://africastalking.com/ussd). The protocol works as follows:

### Request Format

Africa's Talking sends `POST /ussd/callback` with `application/x-www-form-urlencoded`:

| Field | Description |
|---|---|
| `sessionId` | Unique session identifier |
| `phoneNumber` | User's phone in E.164 format (+250781234567) |
| `serviceCode` | USSD shortcode (*384#) |
| `text` | Cumulative user input (see below) |

### Cumulative Text Protocol

The `text` field is cumulative — it contains all user inputs separated by `*`:

| Step | User action | `text` value |
|---|---|---|
| 0 | Dial *384# | `` (empty) |
| 1 | Select "1" (Send Money) | `1` |
| 2 | Enter phone number | `1*0781234567` |
| 3 | Enter amount | `1*0781234567*500` |
| 4 | Enter PIN | `1*0781234567*500*1234` |

### Response Format

Responses are plain text prefixed with:
- `CON ` — session continues, show menu and wait for input
- `END ` — session ends, show final message

### Session State

Sessions are stored in Redis with key `ussd_session:{sessionId}` and a 5-minute TTL. The state includes:
- `current_menu` — which step the user is on
- `phone` — user's phone number
- `branch_data` — context for the current flow (recipient, amount, etc.)

---

## Wallet System

### Key Derivation

Each phone number deterministically maps to a Stellar keypair:

```
master_seed (env: WALLET_MASTER_SEED)
    │
    ▼
HKDF-SHA256(ikm=master_seed, info="stackr-wallet-{phone}")
    │
    ▼
32 bytes → ed25519 signing key → Stellar keypair
```

This is deterministic: the same master seed + phone always produces the same wallet. No seed phrases, no key backup needed by users.

### Encryption at Rest

Keypairs stored in Postgres are encrypted:
- Algorithm: AES-256-GCM
- Key: `WALLET_ENCRYPTION_KEY` (32 bytes / 64 hex chars)
- Nonce: 12 random bytes per record (stored alongside ciphertext)

### PIN Security

- Hashed with Argon2id (memory-hard, GPU/ASIC resistant)
- 4-digit PINs only
- PINs are redacted from all logs

### Critical Warning

**`WALLET_MASTER_SEED` derives every user wallet. Losing it means losing access to all user funds. There is no recovery mechanism.** Back it up with the same care as a root CA private key.

---

## Stellar Integration

### Transaction Builder

`src/services/stellar/tx_builder.rs` constructs Stellar XDR transactions:

- `payment` — USDC transfers between accounts
- `create_account` — fund new accounts with starting XLM balance
- `change_trust` — add USDC trustline during registration
- `path_payment_strict_send` — DEX swaps with automatic path finding
- `fee_bump` — wrap user transactions so the platform pays fees

### Transfer Flow

When a user sends USDC:

1. Look up sender's encrypted keypair from Postgres
2. Decrypt with AES-256-GCM
3. Check idempotency key in Redis (prevents double-send)
4. Build XDR payment operation
5. Sign with sender's key
6. Wrap in fee bump transaction (signed by fee payer)
7. Submit to Stellar Horizon
8. Set idempotency key (1-hour TTL)

### Fee Payer

The `FEE_PAYER_SECRET` account:
- Funds new user accounts with starting XLM balance
- Pays all transaction fees via `FeeBumpTransaction`
- Receives USDC for bill payments (acts as platform treasury)
- Sends refunds back to users when bill API calls fail

Users never need to acquire or hold XLM.

---

## Anchor Integration

### SEP-10 (Authentication)

Before interacting with an anchor, Stackr authenticates via SEP-10:

1. Request challenge from anchor's `WEB_AUTH_ENDPOINT`
2. Receive challenge transaction XDR
3. Sign with user's Stellar key
4. Submit signed transaction back to anchor
5. Receive JWT token for subsequent requests

### SEP-24 (Deposit / Withdrawal)

Deposit flow (fiat → USDC):
1. User selects deposit method (bank/mobile money) and enters amount in local fiat
2. Stackr converts fiat amount to USDC equivalent via price feed
3. Authenticate with anchor via SEP-10
4. Call anchor's SEP-24 deposit endpoint
5. Create anchor order record (status: pending)
6. Anchor collects fiat from user and sends USDC to their Stellar wallet
7. Anchor sends webhook notification → order status updated

Withdrawal flow (USDC → fiat):
1. User selects withdrawal method and enters USDC amount
2. Authenticate with anchor via SEP-10
3. Call anchor's SEP-24 withdrawal endpoint
4. If anchor provides a Stellar address, send USDC to it
5. Anchor converts to fiat and sends to user's bank/mobile money

### Anchor Order Lifecycle

Orders in `anchor_orders` table follow atomic state transitions:

```
pending → processing → completed
                    → error
                    → expired
```

Once an order reaches a terminal state, its status is immutable. This prevents duplicate webhook callbacks from double-crediting users.

---

## Price Feed

`src/services/price_feed.rs` fetches exchange rates from CoinGecko:

- **Endpoint:** `/simple/price?ids=usd-coin&vs_currencies={fiat}`
- **Cache:** Redis with 60-second TTL
- **Supported currencies:** Any currency CoinGecko supports (RWF, NGN, KES, GHS, etc.)

Functions:
- `fiat_to_usdc(state, amount, "rwf")` — convert fiat to USDC equivalent
- `usdc_to_fiat(state, amount, "rwf")` — convert USDC to fiat equivalent
- `usdc_to_fiat_rate(state, "rwf")` — get raw rate (fiat per 1 USDC)
- `xlm_usd_rate(state)` — XLM/USD rate for swap pricing

The `FIAT_CURRENCY` env var determines which currency is used in USSD menus.

---

## Bill Payments

`src/services/bills/airbills.rs` handles airtime, data, and utility payments:

1. User enters amount in local fiat
2. Convert fiat to USDC via price feed
3. Debit USDC from user's wallet (send to fee payer/treasury)
4. Call Airbills API with fiat amount
5. On success: log activity, update daily spend
6. On failure: auto-refund USDC to user from treasury

Supported bill types: airtime, data, electricity, cable TV, betting, internet.

The `FIAT_COUNTRY` env var determines which country is sent to the Airbills API.

---

## Database Schema

6 migrations applied on startup:

### users
| Column | Type | Notes |
|---|---|---|
| phone | TEXT PK | E.164 format |
| public_key | TEXT | Stellar public key |
| encrypted_secret | TEXT | AES-256-GCM encrypted keypair |
| pin_hash | TEXT | Argon2id hash |
| kyc_tier | INT | 0, 1, or 2 |
| daily_spent | NUMERIC | USDC spent today |
| daily_spent_date | DATE | Resets when date changes |
| is_active | BOOL | Soft delete |

### transactions
| Column | Type | Notes |
|---|---|---|
| id | UUID PK | |
| from_phone | TEXT | Sender |
| to_phone | TEXT | Recipient |
| amount | NUMERIC | |
| asset | TEXT | USDC, XLM |
| tx_hash | TEXT | Stellar transaction hash |
| idempotency_key | TEXT | Unique per operation |

### anchor_orders
| Column | Type | Notes |
|---|---|---|
| anchor_tx_id | TEXT PK | From anchor |
| phone | TEXT | User |
| order_type | TEXT | deposit / withdrawal |
| status | TEXT | pending / processing / completed / error / expired |
| asset | TEXT | USDC |
| amount | NUMERIC | USDC amount |
| fiat_amount | NUMERIC | Local fiat amount |
| fiat_currency | TEXT | RWF, NGN, etc. |

### Other tables
- **user_activity** — fire-and-forget analytics ledger
- **merchants** — merchant code → public key mapping
- **ussd_sessions** — session tracking

---

## API Reference

### POST /ussd/callback

Africa's Talking USSD callback. Accepts `application/x-www-form-urlencoded`.

### POST /webhooks/anchor?k={secret}

Anchor webhook receiver. Accepts `application/json`. Secret compared in constant time.

### GET /health

Deep health check. Returns JSON:

```json
{
  "status": "healthy",
  "database": true,
  "redis": true,
  "stellar": true
}
```

---

## Security

### Threat Model

| Threat | Mitigation |
|---|---|
| Brute-force PIN | Argon2id hashing (slow by design) |
| Replay attacks | Redis idempotency keys (1-hour TTL) |
| Webhook forgery | Constant-time secret comparison |
| IP spoofing | X-Forwarded-For only honored from trusted proxies |
| Double-credit | Atomic order state transitions |
| Key compromise | AES-256-GCM encryption at rest |
| Timing attacks | `subtle` crate for all secret comparisons |

### Production Checklist

Before deploying to mainnet:

- [ ] `ENVIRONMENT=production`
- [ ] `WEBHOOK_SECRET` — strong random value
- [ ] `INTERNAL_API_KEY` — strong random value
- [ ] `WALLET_MASTER_SEED` — backed up offline, encrypted
- [ ] `WALLET_ENCRYPTION_KEY` — backed up separately from master seed
- [ ] `FEE_PAYER_SECRET` — funded mainnet account
- [ ] `DATABASE_SSL=true`
- [ ] TLS at reverse proxy (Caddy / nginx / ALB)
- [ ] `TRUSTED_PROXY_IPS` set to proxy IP only
- [ ] `AT_ALLOWED_IPS` set to Africa's Talking production IPs
- [ ] `STELLAR_HORIZON_URL` → mainnet Horizon
- [ ] `STELLAR_NETWORK_PASSPHRASE` → mainnet passphrase
- [ ] `USDC_ISSUER` → mainnet Circle USDC issuer
- [ ] Security audit of wallet derivation and transaction signing
- [ ] Monitoring on failed transactions and refund failures

---

## Production Deployment

### Docker

```bash
docker build -t stackr:latest .
docker run --rm -p 3000:3000 --env-file .env stackr:latest
```

### Reverse Proxy

Terminate TLS at a reverse proxy. Example with Caddy:

```
stackr.example.com {
    reverse_proxy localhost:3000
}
```

Set `TRUSTED_PROXY_IPS` to the proxy's IP so X-Forwarded-For is honored for rate limiting and IP allowlisting.

### Environment Variables

All configuration is via environment variables. See `.env.example` for the complete list with documentation.

---

## Troubleshooting

### Server won't start

- **"Address already in use"** — another process on port 3000. Kill it: `lsof -ti :3000 | xargs kill`
- **"DATABASE_URL required"** — `.env` file missing or not loaded. Make sure it's in the project root.
- **"WALLET_ENCRYPTION_KEY must be exactly 64 hex characters"** — generate one: `openssl rand -hex 32`

### USSD tests fail

- Make sure Postgres and Redis are running
- Make sure the server is running (`cargo run` in another terminal)
- Clear old test data: delete from `users` table if phone numbers conflict

### Stellar operations fail

- **"op_src_no_trust"** — user doesn't have a USDC trustline. Happens if account wasn't funded during registration.
- **"op_underfunded"** — insufficient USDC balance. Fund testnet accounts via Friendbot.
- **"op_already_exists"** — trying to create an account that already exists. Expected on repeat test runs.

### Exchange rate errors

- CoinGecko has rate limits on the free tier. The 60-second Redis cache minimizes API calls.
- If CoinGecko is down, all flows that require fiat conversion will return "Could not get exchange rate."
