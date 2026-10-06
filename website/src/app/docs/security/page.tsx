import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Security",
};

const threats = [
  { threat: "Brute-force PIN", mitigation: "Argon2id hashing (slow by design)" },
  { threat: "Replay attacks", mitigation: "Redis idempotency keys (1-hour TTL)" },
  { threat: "Webhook forgery", mitigation: "Constant-time secret comparison" },
  { threat: "IP spoofing", mitigation: "X-Forwarded-For only honored from trusted proxies" },
  { threat: "Double-credit", mitigation: "Atomic order state transitions" },
  { threat: "Key compromise", mitigation: "AES-256-GCM encryption at rest" },
  { threat: "Timing attacks", mitigation: "subtle crate for all secret comparisons" },
];

const checklist = [
  "ENVIRONMENT=production",
  "WEBHOOK_SECRET — strong random value",
  "INTERNAL_API_KEY — strong random value",
  "WALLET_MASTER_SEED — backed up offline, encrypted",
  "WALLET_ENCRYPTION_KEY — backed up separately from master seed",
  "FEE_PAYER_SECRET — funded mainnet account",
  "DATABASE_SSL=true",
  "TLS at reverse proxy (Caddy / nginx / ALB)",
  "TRUSTED_PROXY_IPS set to proxy IP only",
  "AT_ALLOWED_IPS set to Africa's Talking production IPs",
  "STELLAR_HORIZON_URL → mainnet Horizon",
  "STELLAR_NETWORK_PASSPHRASE → mainnet passphrase",
  "USDC_ISSUER → mainnet Circle USDC issuer",
  "Security audit of wallet derivation and transaction signing",
  "Monitoring on failed transactions and refund failures",
];

export default function SecurityPage() {
  return (
    <div className="max-w-none">
      <h1 className="text-3xl font-bold tracking-tight mb-2">Security</h1>
      <p className="text-zinc-400 text-lg mb-10">
        Threat model, cryptographic primitives, and production hardening checklist.
      </p>

      <section className="mb-16">
        <h2 className="text-2xl font-bold mb-4">Threat Model</h2>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-zinc-800">
                <th className="text-left py-2 pr-6 text-zinc-400 font-medium">Threat</th>
                <th className="text-left py-2 text-zinc-400 font-medium">Mitigation</th>
              </tr>
            </thead>
            <tbody>
              {threats.map(({ threat, mitigation }) => (
                <tr key={threat} className="border-b border-zinc-800/50">
                  <td className="py-2.5 pr-6 text-zinc-300">{threat}</td>
                  <td className="py-2.5 text-zinc-400">{mitigation}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      <section className="mb-16">
        <h2 className="text-2xl font-bold mb-4">Cryptography</h2>
        <div className="grid sm:grid-cols-2 gap-4">
          {[
            { title: "Wallet Key Derivation", desc: "HKDF-SHA256 from master seed + phone number → deterministic ed25519 keypair" },
            { title: "Encryption at Rest", desc: "AES-256-GCM with 12-byte random nonce per record. Key from WALLET_ENCRYPTION_KEY env var." },
            { title: "PIN Hashing", desc: "Argon2id — memory-hard, resistant to GPU and ASIC attacks. 4-digit PINs only." },
            { title: "Secret Comparisons", desc: "All authentication checks use the subtle crate for constant-time comparison." },
          ].map(({ title, desc }) => (
            <div key={title} className="p-4 rounded-lg border border-zinc-800 bg-zinc-900/30">
              <h3 className="text-sm font-semibold text-white mb-1">{title}</h3>
              <p className="text-xs text-zinc-400 leading-relaxed">{desc}</p>
            </div>
          ))}
        </div>
      </section>

      <section>
        <h2 className="text-2xl font-bold mb-4">Production Checklist</h2>
        <p className="text-sm text-zinc-400 mb-4">Before deploying to mainnet:</p>
        <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4">
          <ul className="space-y-2">
            {checklist.map((item) => (
              <li key={item} className="flex items-start gap-2 text-sm">
                <span className="text-zinc-600 mt-0.5">&#9744;</span>
                <span className="text-zinc-400">{item}</span>
              </li>
            ))}
          </ul>
        </div>
      </section>
    </div>
  );
}
