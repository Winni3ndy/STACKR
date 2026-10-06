import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Self-Hosting",
};

const envVars = [
  { name: "HOST", default: "0.0.0.0", desc: "Listen address" },
  { name: "PORT", default: "3000", desc: "Listen port" },
  { name: "ENVIRONMENT", default: "development", desc: "development | staging | production" },
  { name: "DATABASE_URL", default: "—", desc: "Postgres connection string" },
  { name: "REDIS_URL", default: "redis://127.0.0.1:6379", desc: "Redis connection string" },
  { name: "STELLAR_HORIZON_URL", default: "testnet", desc: "Horizon API endpoint" },
  { name: "FEE_PAYER_SECRET", default: "—", desc: "Stellar account that pays fees and funds new accounts" },
  { name: "USDC_ASSET_CODE", default: "USDC", desc: "Asset code on Stellar" },
  { name: "USDC_ISSUER", default: "testnet issuer", desc: "USDC issuer account" },
  { name: "WALLET_MASTER_SEED", default: "—", desc: "Master seed for wallet derivation (CRITICAL)" },
  { name: "WALLET_ENCRYPTION_KEY", default: "—", desc: "64 hex chars for AES-256-GCM" },
  { name: "FIAT_CURRENCY", default: "RWF", desc: "Local fiat currency code" },
  { name: "FIAT_COUNTRY", default: "RW", desc: "Country code for Airbills API" },
  { name: "ANCHOR_DOMAIN", default: "—", desc: "SEP-24 anchor domain" },
  { name: "WEBHOOK_SECRET", default: "—", desc: "Anchor webhook authentication" },
  { name: "INTERNAL_API_KEY", default: "—", desc: "API v1 authentication key" },
];

export default function DeploymentPage() {
  return (
    <div className="max-w-none">
      <h1 className="text-3xl font-bold tracking-tight mb-2">Deployment</h1>
      <p className="text-zinc-400 text-lg mb-10">
        Docker, reverse proxy setup, and environment variable reference.
      </p>

      <section className="mb-16">
        <h2 className="text-2xl font-bold mb-4">Docker</h2>
        <div className="space-y-4">
          <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4">
            <pre className="font-mono text-sm text-zinc-300">
{`# Build
docker build -t stackr:latest .

# Run
docker run --rm -p 3000:3000 --env-file .env stackr:latest

# Or use docker-compose
docker-compose up -d`}
            </pre>
          </div>
        </div>
      </section>

      <section className="mb-16">
        <h2 className="text-2xl font-bold mb-4">Reverse Proxy</h2>
        <p className="text-sm text-zinc-400 mb-4">
          Terminate TLS at a reverse proxy. Set <code className="text-zinc-300 bg-zinc-800 px-1.5 py-0.5 rounded text-xs">TRUSTED_PROXY_IPS</code> to the proxy&apos;s IP.
        </p>
        <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4">
          <p className="text-xs text-zinc-500 mb-2">Caddyfile</p>
          <pre className="font-mono text-sm text-zinc-300">
{`stackr.example.com {
    reverse_proxy localhost:3000
}`}
          </pre>
        </div>
      </section>

      <section className="mb-16">
        <h2 className="text-2xl font-bold mb-4">Environment Variables</h2>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-zinc-800">
                <th className="text-left py-2 pr-4 text-zinc-400 font-medium">Variable</th>
                <th className="text-left py-2 pr-4 text-zinc-400 font-medium">Default</th>
                <th className="text-left py-2 text-zinc-400 font-medium">Description</th>
              </tr>
            </thead>
            <tbody>
              {envVars.map(({ name, default: def, desc }) => (
                <tr key={name} className="border-b border-zinc-800/50">
                  <td className="py-2 pr-4">
                    <code className="text-green-500 text-xs">{name}</code>
                  </td>
                  <td className="py-2 pr-4 text-zinc-500 text-xs font-mono">{def}</td>
                  <td className="py-2 text-zinc-400">{desc}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      <section>
        <h2 className="text-2xl font-bold mb-4">Troubleshooting</h2>
        <div className="space-y-4">
          {[
            {
              q: "\"Address already in use\"",
              a: "Another process on port 3000. Kill it: lsof -ti :3000 | xargs kill",
            },
            {
              q: "\"DATABASE_URL required\"",
              a: ".env file missing or not loaded. Ensure it's in the project root.",
            },
            {
              q: "\"WALLET_ENCRYPTION_KEY must be exactly 64 hex characters\"",
              a: "Generate one: openssl rand -hex 32",
            },
            {
              q: "Stellar \"op_src_no_trust\"",
              a: "User doesn't have a USDC trustline. Account wasn't funded during registration.",
            },
            {
              q: "Exchange rate errors",
              a: "CoinGecko rate limits on free tier. The 60-second Redis cache minimizes API calls.",
            },
          ].map(({ q, a }) => (
            <div key={q} className="p-4 rounded-lg border border-zinc-800 bg-zinc-900/30">
              <p className="text-sm font-semibold text-white mb-1">{q}</p>
              <p className="text-sm text-zinc-400">{a}</p>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}
