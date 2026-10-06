Stellar SCF Application Answers


Current Traction

Stackr is a working product, not just an idea. The full backend and website are built, deployed, and live right now.

What we have built and shipped so far:

The backend is a complete Rust server that handles everything from creating wallets to sending money to paying bills. It is deployed on a VPS and running 24/7 at api.usestackr.xyz. You can hit the health check right now and it will respond. The database, cache, and Stellar connection are all live and healthy.

The website is live at usestackr.xyz. It has a landing page, documentation with curl examples for every endpoint, a pricing page, a working signup and login system powered by Supabase, and a developer dashboard where users can create and manage API keys. The API keys are generated using cryptographically secure randomness and stored as SHA-256 hashes so even if the database leaks, the keys are safe.

The API has 17 REST endpoints covering account creation, USDC transfers, balance checks, exchange rates, transaction history, deposits, withdrawals, merchant payments, airtime purchases, utility bill payments, and token swaps. Every endpoint has proper authentication, rate limiting, and input validation. The full API reference with request and response examples is at docs.usestackr.xyz.

The USSD flow is fully built. A user dials a shortcode and gets a menu. They can send money, pay a merchant, buy airtime, pay electricity or cable bills, check their balance, swap between USDC and XLM, deposit money in, or withdraw money out. The entire flow works through text menus on any phone, even a basic feature phone on a 2G network. We built an interactive USSD simulator on the website so people can try the full flow from their browser without needing a real phone.

Where we are right now:

We are in pre-launch stage. The product is fully built and deployed. We are on Stellar testnet, not mainnet yet. We have not onboarded real users because we are waiting for Africa's Talking to approve our production USSD shortcode and for our SEP-24 anchor integration to go live for real fiat deposits and withdrawals. Once those two things are in place, we flip the config from testnet to mainnet and real money starts flowing.

The codebase is open source at github.com/Winni3ndy/STACKR.

We support phone number formats for four countries out of the box: Rwanda, Nigeria, Kenya, and Ghana. The system is designed so adding a new country is just a config change for the currency code and country code, not a code rewrite.

Evidence:

Live website: https://usestackr.xyz
Live API: https://api.usestackr.xyz/health
API documentation: https://docs.usestackr.xyz
Source code: https://github.com/Winni3ndy/STACKR
USSD simulator: https://usestackr.xyz/how-it-works


Planned Stellar Integration

Stackr is not planning to integrate with Stellar someday. It already runs on Stellar. Every transaction in the system is a real Stellar transaction. Here is exactly how we use the Stellar tech stack today.

Wallets: When someone signs up with their phone number, Stackr creates a real Stellar account for them on the network. We derive a unique keypair for each phone number using HKDF-SHA256 from a master seed. The private keys are encrypted with AES-256-GCM and stored in our database. The system then funds the new account with XLM from a fee payer account and adds a USDC trustline so the wallet is ready to receive stablecoins immediately.

Payments: When a user sends money to another phone number, Stackr builds a real Stellar payment transaction. It looks up the recipient's Stellar public key, constructs the XDR transaction envelope, signs it with the sender's private key, and submits it to Horizon. The money moves as USDC on the Stellar network. Settlement is final in about 5 seconds.

Fee bumping: Every user transaction is wrapped in a fee bump transaction paid by our platform fee payer account. Users never need to worry about having XLM for fees. The platform covers all network fees.

Token swaps: Users can swap between USDC and XLM directly from the USSD menu. We use Stellar's path_payment_strict_send operation which routes through the Stellar DEX to find the best exchange rate automatically.

Deposits and withdrawals: We use SEP-24, which is Stellar's standard protocol for moving money between fiat and crypto. When a user wants to deposit money from their mobile money or bank account, we authenticate with the anchor using SEP-10 (challenge-response authentication), then initiate a SEP-24 interactive deposit. The anchor handles the fiat side and sends USDC to the user's Stellar wallet. Withdrawals work the same way in reverse. We listen for status updates through a webhook endpoint. Any SEP-24 compliant anchor works with Stackr through a config change, no code changes needed.

Exchange rates: We fetch live USDC-to-local-currency rates from CoinGecko and cache them in Redis for 60 seconds. This powers the conversion display when users see amounts in their local currency like Rwandan Francs or Nigerian Naira.

Idempotency: Every money movement has a Redis-backed idempotency key so the same transfer cannot be processed twice, even if the user double-taps on their phone or the network retries the request.

What we plan to build next on Stellar:

We want to add support for USDT and other Stellar assets beyond just USDC, so users have more stablecoin options. We also plan to integrate Stellar's Soroban smart contracts for features like scheduled recurring payments and escrow for merchant disputes. And we want to connect with more SEP-24 anchors across different African countries so users have multiple on-ramp and off-ramp options for their local currency.
