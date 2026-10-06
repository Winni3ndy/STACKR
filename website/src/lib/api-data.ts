export interface ApiEndpoint {
  method: "GET" | "POST";
  path: string;
  title: string;
  description: string;
  auth: boolean;
  curl: string;
  request?: {
    contentType: string;
    body: string;
  };
  response: {
    status: number;
    body: string;
  };
  errors: { status: number; description: string }[];
}

export const endpoints: ApiEndpoint[] = [
  {
    method: "POST",
    path: "/ussd/callback",
    title: "USSD Callback",
    description:
      "Africa's Talking USSD callback. Receives user input, returns menu text. Protected by IP allowlist and rate limiting.",
    auth: false,
    curl: `curl -X POST http://localhost:3000/ussd/callback \\
  -H "Content-Type: application/x-www-form-urlencoded" \\
  -d "sessionId=sim-1234567890&phoneNumber=%2B250781234567&serviceCode=*384%23&text=1"`,
    request: {
      contentType: "application/x-www-form-urlencoded",
      body: `sessionId=sim-1234567890
phoneNumber=+250781234567
serviceCode=*384#
text=1*0781234567*500`,
    },
    response: {
      status: 200,
      body: `CON Confirm transfer:
To: +250781234567
Amount: 500 USDC

Enter PIN to confirm:`,
    },
    errors: [
      { status: 403, description: "IP not in AT_ALLOWED_IPS allowlist" },
      { status: 429, description: "Rate limit exceeded (default: 60 req/min)" },
    ],
  },
  {
    method: "GET",
    path: "/health",
    title: "Health Check",
    description:
      "Deep health check that verifies database, Redis, and Stellar connectivity. Use for monitoring and load balancer health probes.",
    auth: false,
    curl: `curl http://localhost:3000/health`,
    response: {
      status: 200,
      body: JSON.stringify(
        { status: "healthy", version: "0.1.0", database: true, redis: true, stellar: true },
        null, 2
      ),
    },
    errors: [
      { status: 503, description: "One or more services unhealthy — check database/redis/stellar fields" },
    ],
  },
  {
    method: "POST",
    path: "/webhooks/anchor?k={secret}",
    title: "Anchor Webhook",
    description:
      "SEP-24 anchor webhook for deposit/withdrawal status updates. The k query parameter must match WEBHOOK_SECRET (constant-time comparison).",
    auth: false,
    curl: `curl -X POST "http://localhost:3000/webhooks/anchor?k=your_webhook_secret" \\
  -H "Content-Type: application/json" \\
  -d '{"id":"txn_abc123","status":"completed","stellar_transaction_id":"3d8f...a91c","amount_in":"50.00","amount_out":"49.50"}'`,
    request: {
      contentType: "application/json",
      body: JSON.stringify(
        { id: "txn_abc123", status: "completed", stellar_transaction_id: "3d8f...a91c", amount_in: "50.00", amount_out: "49.50" },
        null, 2
      ),
    },
    response: { status: 200, body: "" },
    errors: [
      { status: 401, description: "Missing or invalid webhook secret" },
    ],
  },
  {
    method: "POST",
    path: "/api/v1/accounts",
    title: "Create Account",
    description:
      "Register a new user. Derives a Stellar keypair from the phone number, funds the account with starting XLM, and adds a USDC trustline.",
    auth: true,
    curl: `curl -X POST http://localhost:3000/api/v1/accounts \\
  -H "Content-Type: application/json" \\
  -H "Authorization: Bearer \$INTERNAL_API_KEY" \\
  -d '{"phone":"+250781234567","pin":"1234"}'`,
    request: {
      contentType: "application/json",
      body: JSON.stringify({ phone: "+250781234567", pin: "1234" }, null, 2),
    },
    response: {
      status: 201,
      body: JSON.stringify(
        { id: "+250781234567", phone: "+250781234567", public_key: "GBXYZ2KCJQPG3RV7PMHZCFN4AOXK4NP..." },
        null, 2
      ),
    },
    errors: [
      { status: 400, description: "Invalid phone format or PIN not 4 digits" },
      { status: 401, description: "Missing or invalid API key" },
      { status: 409, description: "Account already exists for this phone number" },
    ],
  },
  {
    method: "GET",
    path: "/api/v1/accounts/{phone}",
    title: "Get Account",
    description: "Retrieve account details including Stellar public key, KYC tier, and active status.",
    auth: true,
    curl: `curl http://localhost:3000/api/v1/accounts/%2B250781234567 \\
  -H "Authorization: Bearer \$INTERNAL_API_KEY"`,
    response: {
      status: 200,
      body: JSON.stringify(
        { id: "+250781234567", phone: "+250781234567", public_key: "GBXYZ2KCJQPG3RV7PMHZCFN4AOXK4NP...", kyc_tier: 1, is_active: true, created_at: "2024-01-15T10:30:00Z" },
        null, 2
      ),
    },
    errors: [
      { status: 401, description: "Missing or invalid API key" },
      { status: 404, description: "No account found for this phone number" },
    ],
  },
  {
    method: "GET",
    path: "/api/v1/accounts/{phone}/balance",
    title: "Get Balance",
    description: "Fetch USDC and XLM balances directly from the Stellar network (not cached).",
    auth: true,
    curl: `curl http://localhost:3000/api/v1/accounts/%2B250781234567/balance \\
  -H "Authorization: Bearer \$INTERNAL_API_KEY"`,
    response: {
      status: 200,
      body: JSON.stringify(
        { balances: [{ asset: "USDC", amount: "250.00" }, { asset: "XLM", amount: "5.00" }] },
        null, 2
      ),
    },
    errors: [
      { status: 401, description: "Missing or invalid API key" },
      { status: 404, description: "Account not found" },
      { status: 502, description: "Failed to reach Stellar Horizon" },
    ],
  },
  {
    method: "POST",
    path: "/api/v1/transfers",
    title: "Create Transfer",
    description:
      "Send USDC between accounts. Verifies PIN, checks KYC daily limits, builds and submits a fee-bumped Stellar transaction.",
    auth: true,
    curl: `curl -X POST http://localhost:3000/api/v1/transfers \\
  -H "Content-Type: application/json" \\
  -H "Authorization: Bearer \$INTERNAL_API_KEY" \\
  -d '{"sender_phone":"+250781234567","recipient_phone":"+250789876543","amount":"10.00","pin":"1234"}'`,
    request: {
      contentType: "application/json",
      body: JSON.stringify(
        { sender_phone: "+250781234567", recipient_phone: "+250789876543", amount: "10.00", pin: "1234" },
        null, 2
      ),
    },
    response: {
      status: 200,
      body: JSON.stringify(
        { tx_hash: "3d8f4a2b1c9e8d7f6a5b4c3d2e1f0a9b8c7d6e5f", status: "completed" },
        null, 2
      ),
    },
    errors: [
      { status: 400, description: "Invalid amount, missing fields, or amount exceeds balance" },
      { status: 401, description: "Missing or invalid API key" },
      { status: 403, description: "Incorrect PIN or daily limit exceeded" },
      { status: 404, description: "Sender or recipient account not found" },
    ],
  },
  {
    method: "GET",
    path: "/api/v1/rates",
    title: "Get Exchange Rates",
    description:
      "Current USDC-to-fiat exchange rate from CoinGecko. Cached in Redis for 60 seconds. The fiat currency is set by the FIAT_CURRENCY env var.",
    auth: true,
    curl: `curl http://localhost:3000/api/v1/rates \\
  -H "Authorization: Bearer \$INTERNAL_API_KEY"`,
    response: {
      status: 200,
      body: JSON.stringify({ usdc_to_fiat: 1370.25, fiat_currency: "RWF" }, null, 2),
    },
    errors: [
      { status: 401, description: "Missing or invalid API key" },
      { status: 502, description: "CoinGecko API unreachable and no cached rate available" },
    ],
  },
  {
    method: "GET",
    path: "/api/v1/transactions",
    title: "List Transactions",
    description:
      "Paginated transaction history for a phone number. Query params: phone (required), limit (optional, default 20, max 100).",
    auth: true,
    curl: `curl "http://localhost:3000/api/v1/transactions?phone=%2B250781234567&limit=20" \\
  -H "Authorization: Bearer \$INTERNAL_API_KEY"`,
    response: {
      status: 200,
      body: JSON.stringify(
        {
          transactions: [{
            id: "a1b2c3d4-e5f6-7890-abcd-ef1234567890", tx_type: "transfer", status: "completed",
            amount: "10.00", asset_code: "USDC", stellar_tx_hash: "3d8f...a91c",
            recipient_phone: "+250789876543", created_at: "2024-01-15T14:30:00Z",
          }],
        },
        null, 2
      ),
    },
    errors: [
      { status: 400, description: "Missing phone parameter or invalid limit" },
      { status: 401, description: "Missing or invalid API key" },
    ],
  },
  {
    method: "POST",
    path: "/api/v1/deposits",
    title: "Create Deposit",
    description:
      "Initiate a fiat-to-USDC deposit via SEP-24 anchor. Returns a redirect URL where the user completes the fiat payment.",
    auth: true,
    curl: `curl -X POST http://localhost:3000/api/v1/deposits \\
  -H "Content-Type: application/json" \\
  -H "Authorization: Bearer \$INTERNAL_API_KEY" \\
  -d '{"phone":"+250781234567","pin":"1234","amount_fiat":50000}'`,
    request: {
      contentType: "application/json",
      body: JSON.stringify({ phone: "+250781234567", pin: "1234", amount_fiat: 50000 }, null, 2),
    },
    response: {
      status: 200,
      body: JSON.stringify(
        { id: "dep_abc123", status: "pending", url: "https://anchor.example.com/deposit/abc123" },
        null, 2
      ),
    },
    errors: [
      { status: 401, description: "Missing or invalid API key" },
      { status: 403, description: "Incorrect PIN" },
      { status: 502, description: "Anchor unreachable or SEP-10 auth failed" },
    ],
  },
  {
    method: "GET",
    path: "/api/v1/deposits/{id}",
    title: "Get Deposit Status",
    description: "Check the status of a deposit order. Status transitions: pending → processing → completed | error | expired.",
    auth: true,
    curl: `curl http://localhost:3000/api/v1/deposits/dep_abc123 \\
  -H "Authorization: Bearer \$INTERNAL_API_KEY"`,
    response: {
      status: 200,
      body: JSON.stringify(
        { id: "dep_abc123", status: "completed", direction: "deposit", amount: "36.50", fiat_amount: "50000", fiat_currency: "RWF", stellar_tx_hash: "9f2a...c4d1" },
        null, 2
      ),
    },
    errors: [
      { status: 401, description: "Missing or invalid API key" },
      { status: 404, description: "Deposit order not found" },
    ],
  },
  {
    method: "POST",
    path: "/api/v1/withdrawals",
    title: "Create Withdrawal",
    description:
      "Initiate a USDC-to-fiat withdrawal via SEP-24 anchor. If the anchor provides a Stellar address, USDC is sent immediately.",
    auth: true,
    curl: `curl -X POST http://localhost:3000/api/v1/withdrawals \\
  -H "Content-Type: application/json" \\
  -H "Authorization: Bearer \$INTERNAL_API_KEY" \\
  -d '{"phone":"+250781234567","pin":"1234","amount":"50.00"}'`,
    request: {
      contentType: "application/json",
      body: JSON.stringify({ phone: "+250781234567", pin: "1234", amount: "50.00" }, null, 2),
    },
    response: {
      status: 200,
      body: JSON.stringify(
        { id: "wd_xyz789", status: "pending", url: "https://anchor.example.com/withdraw/xyz789" },
        null, 2
      ),
    },
    errors: [
      { status: 400, description: "Amount exceeds balance" },
      { status: 401, description: "Missing or invalid API key" },
      { status: 403, description: "Incorrect PIN or daily limit exceeded" },
      { status: 502, description: "Anchor unreachable" },
    ],
  },
  {
    method: "GET",
    path: "/api/v1/withdrawals/{id}",
    title: "Get Withdrawal Status",
    description: "Check the status of a withdrawal order. Status transitions: pending → processing → completed | error | expired.",
    auth: true,
    curl: `curl http://localhost:3000/api/v1/withdrawals/wd_xyz789 \\
  -H "Authorization: Bearer \$INTERNAL_API_KEY"`,
    response: {
      status: 200,
      body: JSON.stringify(
        { id: "wd_xyz789", status: "completed", direction: "withdrawal", amount: "50.00", fiat_amount: "68500", fiat_currency: "RWF", stellar_tx_hash: "7b3e...d8f2" },
        null, 2
      ),
    },
    errors: [
      { status: 401, description: "Missing or invalid API key" },
      { status: 404, description: "Withdrawal order not found" },
    ],
  },
  {
    method: "POST",
    path: "/api/v1/payments/merchant",
    title: "Pay Merchant",
    description:
      "Pay a registered merchant by code. Looks up the merchant's Stellar address and sends USDC instantly.",
    auth: true,
    curl: `curl -X POST http://localhost:3000/api/v1/payments/merchant \\
  -H "Content-Type: application/json" \\
  -H "Authorization: Bearer \$INTERNAL_API_KEY" \\
  -d '{"sender_phone":"+250781234567","merchant_code":"QM-001","amount":"5.00","pin":"1234"}'`,
    request: {
      contentType: "application/json",
      body: JSON.stringify(
        { sender_phone: "+250781234567", merchant_code: "QM-001", amount: "5.00", pin: "1234" },
        null, 2
      ),
    },
    response: {
      status: 200,
      body: JSON.stringify(
        { tx_hash: "4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b", status: "completed", merchant_name: "QuickMart" },
        null, 2
      ),
    },
    errors: [
      { status: 401, description: "Missing or invalid API key" },
      { status: 403, description: "Incorrect PIN" },
      { status: 404, description: "Merchant code not found" },
    ],
  },
  {
    method: "POST",
    path: "/api/v1/payments/airtime",
    title: "Buy Airtime",
    description:
      "Purchase airtime or data. Debits USDC from user's wallet, calls Airbills API with fiat amount. Auto-refunds USDC on provider failure.",
    auth: true,
    curl: `curl -X POST http://localhost:3000/api/v1/payments/airtime \\
  -H "Content-Type: application/json" \\
  -H "Authorization: Bearer \$INTERNAL_API_KEY" \\
  -d '{"phone":"+250781234567","target_phone":"+250781234567","product_type":"airtime","amount_fiat":1000,"pin":"1234"}'`,
    request: {
      contentType: "application/json",
      body: JSON.stringify(
        { phone: "+250781234567", target_phone: "+250781234567", product_type: "airtime", amount_fiat: 1000, pin: "1234" },
        null, 2
      ),
    },
    response: {
      status: 200,
      body: JSON.stringify({ reference: "AIR-28491", status: "completed" }, null, 2),
    },
    errors: [
      { status: 400, description: "Invalid product_type or amount" },
      { status: 401, description: "Missing or invalid API key" },
      { status: 403, description: "Incorrect PIN or daily limit exceeded" },
      { status: 502, description: "Airbills API failure (USDC auto-refunded)" },
    ],
  },
  {
    method: "POST",
    path: "/api/v1/payments/bill",
    title: "Pay Bill",
    description:
      "Pay utility bills (electricity, cable TV, internet, betting). Debits USDC, calls Airbills API. Auto-refunds on failure.",
    auth: true,
    curl: `curl -X POST http://localhost:3000/api/v1/payments/bill \\
  -H "Content-Type: application/json" \\
  -H "Authorization: Bearer \$INTERNAL_API_KEY" \\
  -d '{"phone":"+250781234567","bill_type":"electricity","account_number":"04829184729","amount_fiat":5000,"pin":"1234"}'`,
    request: {
      contentType: "application/json",
      body: JSON.stringify(
        { phone: "+250781234567", bill_type: "electricity", account_number: "04829184729", amount_fiat: 5000, pin: "1234" },
        null, 2
      ),
    },
    response: {
      status: 200,
      body: JSON.stringify({ reference: "BILL-91827", status: "completed" }, null, 2),
    },
    errors: [
      { status: 400, description: "Invalid bill_type or account_number" },
      { status: 401, description: "Missing or invalid API key" },
      { status: 403, description: "Incorrect PIN or daily limit exceeded" },
      { status: 502, description: "Airbills API failure (USDC auto-refunded)" },
    ],
  },
  {
    method: "POST",
    path: "/api/v1/swaps",
    title: "Swap Tokens",
    description:
      "Swap between USDC and XLM using Stellar DEX path payments. Uses path_payment_strict_send for predictable input amounts.",
    auth: true,
    curl: `curl -X POST http://localhost:3000/api/v1/swaps \\
  -H "Content-Type: application/json" \\
  -H "Authorization: Bearer \$INTERNAL_API_KEY" \\
  -d '{"phone":"+250781234567","source_asset":"USDC","dest_asset":"XLM","amount":"10.00","pin":"1234"}'`,
    request: {
      contentType: "application/json",
      body: JSON.stringify(
        { phone: "+250781234567", source_asset: "USDC", dest_asset: "XLM", amount: "10.00", pin: "1234" },
        null, 2
      ),
    },
    response: {
      status: 200,
      body: JSON.stringify(
        { tx_hash: "5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d", status: "completed", sent_amount: "10.00", sent_asset: "USDC", received_amount: "85.47", received_asset: "XLM" },
        null, 2
      ),
    },
    errors: [
      { status: 400, description: "Invalid asset pair or amount exceeds balance" },
      { status: 401, description: "Missing or invalid API key" },
      { status: 403, description: "Incorrect PIN" },
      { status: 502, description: "No swap path found on Stellar DEX" },
    ],
  },
];
