import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Developer Guide",
};

export default function GuidePage() {
  return (
    <div className="max-w-none">
      <h1 className="text-3xl font-bold tracking-tight mb-2">Developer Guide</h1>
      <p className="text-zinc-400 text-lg mb-10">
        How Stackr works under the hood — USSD protocol, wallet system, and Stellar integration.
      </p>

      {/* USSD Protocol */}
      <section className="mb-16">
        <h2 className="text-2xl font-bold mb-4">USSD Protocol</h2>
        <p className="text-zinc-400 mb-6">
          Stackr uses the Africa&apos;s Talking USSD API. The protocol is text-based: Africa&apos;s Talking sends a POST request for each user interaction, and Stackr responds with menu text.
        </p>

        <h3 className="text-lg font-semibold mb-3">Request Format</h3>
        <p className="text-sm text-zinc-400 mb-3">
          Africa&apos;s Talking sends <code className="text-zinc-300 bg-zinc-800 px-1.5 py-0.5 rounded text-xs">POST /ussd/callback</code> with <code className="text-zinc-300 bg-zinc-800 px-1.5 py-0.5 rounded text-xs">application/x-www-form-urlencoded</code>:
        </p>

        <div className="overflow-x-auto mb-6">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-zinc-800">
                <th className="text-left py-2 pr-4 text-zinc-400 font-medium">Field</th>
                <th className="text-left py-2 text-zinc-400 font-medium">Description</th>
              </tr>
            </thead>
            <tbody className="text-zinc-400">
              <tr className="border-b border-zinc-800/50">
                <td className="py-2 pr-4"><code className="text-green-500 text-xs">sessionId</code></td>
                <td className="py-2">Unique session identifier</td>
              </tr>
              <tr className="border-b border-zinc-800/50">
                <td className="py-2 pr-4"><code className="text-green-500 text-xs">phoneNumber</code></td>
                <td className="py-2">User&apos;s phone in E.164 format (+250781234567)</td>
              </tr>
              <tr className="border-b border-zinc-800/50">
                <td className="py-2 pr-4"><code className="text-green-500 text-xs">serviceCode</code></td>
                <td className="py-2">USSD shortcode (*384#)</td>
              </tr>
              <tr>
                <td className="py-2 pr-4"><code className="text-green-500 text-xs">text</code></td>
                <td className="py-2">Cumulative user input separated by *</td>
              </tr>
            </tbody>
          </table>
        </div>

        <h3 className="text-lg font-semibold mb-3">Cumulative Text Protocol</h3>
        <p className="text-sm text-zinc-400 mb-3">
          The <code className="text-zinc-300 bg-zinc-800 px-1.5 py-0.5 rounded text-xs">text</code> field accumulates all user inputs:
        </p>

        <div className="overflow-x-auto mb-6">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-zinc-800">
                <th className="text-left py-2 pr-4 text-zinc-400 font-medium">Step</th>
                <th className="text-left py-2 pr-4 text-zinc-400 font-medium">User action</th>
                <th className="text-left py-2 text-zinc-400 font-medium">text value</th>
              </tr>
            </thead>
            <tbody className="text-zinc-400 font-mono text-xs">
              <tr className="border-b border-zinc-800/50">
                <td className="py-2 pr-4">0</td>
                <td className="py-2 pr-4 font-sans">Dial *384#</td>
                <td className="py-2 text-zinc-600">(empty)</td>
              </tr>
              <tr className="border-b border-zinc-800/50">
                <td className="py-2 pr-4">1</td>
                <td className="py-2 pr-4 font-sans">Select &quot;1&quot; (Send Money)</td>
                <td className="py-2 text-green-500">1</td>
              </tr>
              <tr className="border-b border-zinc-800/50">
                <td className="py-2 pr-4">2</td>
                <td className="py-2 pr-4 font-sans">Enter phone</td>
                <td className="py-2 text-green-500">1*0781234567</td>
              </tr>
              <tr className="border-b border-zinc-800/50">
                <td className="py-2 pr-4">3</td>
                <td className="py-2 pr-4 font-sans">Enter amount</td>
                <td className="py-2 text-green-500">1*0781234567*500</td>
              </tr>
              <tr>
                <td className="py-2 pr-4">4</td>
                <td className="py-2 pr-4 font-sans">Enter PIN</td>
                <td className="py-2 text-green-500">1*0781234567*500*1234</td>
              </tr>
            </tbody>
          </table>
        </div>

        <h3 className="text-lg font-semibold mb-3">Response Format</h3>
        <p className="text-sm text-zinc-400 mb-3">Responses are plain text prefixed with:</p>
        <ul className="space-y-2 text-sm text-zinc-400 mb-6">
          <li><code className="text-green-500 bg-zinc-800 px-1.5 py-0.5 rounded text-xs">CON </code> — session continues, show menu and wait for input</li>
          <li><code className="text-amber-400 bg-zinc-800 px-1.5 py-0.5 rounded text-xs">END </code> — session ends, show final message</li>
        </ul>
      </section>

      {/* Wallet System */}
      <section className="mb-16">
        <h2 className="text-2xl font-bold mb-4">Wallet System</h2>

        <h3 className="text-lg font-semibold mb-3">Key Derivation</h3>
        <p className="text-sm text-zinc-400 mb-4">
          Each phone number deterministically maps to a Stellar keypair. The same master seed + phone always produces the same wallet.
        </p>

        <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4 mb-6">
          <pre className="font-mono text-sm text-zinc-400 leading-relaxed">
{`master_seed (env: WALLET_MASTER_SEED)
    │
    ▼
HKDF-SHA256(ikm=master_seed, info="stackr-wallet-{phone}")
    │
    ▼
32 bytes → ed25519 signing key → Stellar keypair`}
          </pre>
        </div>

        <h3 className="text-lg font-semibold mb-3">Security Layers</h3>
        <div className="grid sm:grid-cols-2 gap-4 mb-6">
          {[
            { title: "Encryption at rest", desc: "AES-256-GCM with 12-byte random nonce per record" },
            { title: "PIN hashing", desc: "Argon2id — memory-hard, GPU/ASIC resistant" },
            { title: "Key derivation", desc: "HKDF-SHA256 from master seed + phone" },
            { title: "Fee payer", desc: "Users never hold or acquire XLM" },
          ].map(({ title, desc }) => (
            <div key={title} className="p-4 rounded-lg border border-zinc-800 bg-zinc-900/30">
              <h4 className="text-sm font-semibold text-white mb-1">{title}</h4>
              <p className="text-xs text-zinc-400">{desc}</p>
            </div>
          ))}
        </div>

        <div className="rounded-lg border border-amber-900/50 bg-amber-950/20 p-4">
          <p className="text-sm text-amber-400 font-semibold mb-1">Critical Warning</p>
          <p className="text-sm text-amber-400/80">
            WALLET_MASTER_SEED derives every user wallet. Losing it means losing access to all user funds. There is no recovery mechanism. Back it up with the same care as a root CA private key.
          </p>
        </div>
      </section>

      {/* Stellar Integration */}
      <section className="mb-16">
        <h2 className="text-2xl font-bold mb-4">Stellar Integration</h2>

        <h3 className="text-lg font-semibold mb-3">Transfer Flow</h3>
        <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4 mb-6">
          <ol className="space-y-2 text-sm text-zinc-400">
            <li>1. Look up sender&apos;s encrypted keypair from Postgres</li>
            <li>2. Decrypt with AES-256-GCM</li>
            <li>3. Check idempotency key in Redis (prevents double-send)</li>
            <li>4. Build XDR payment operation</li>
            <li>5. Sign with sender&apos;s key</li>
            <li>6. Wrap in fee bump transaction (signed by fee payer)</li>
            <li>7. Submit to Stellar Horizon</li>
            <li>8. Set idempotency key (1-hour TTL)</li>
          </ol>
        </div>

        <h3 className="text-lg font-semibold mb-3">Transaction Types</h3>
        <ul className="space-y-1 text-sm text-zinc-400">
          <li><code className="text-green-500 text-xs">payment</code> — USDC transfers between accounts</li>
          <li><code className="text-green-500 text-xs">create_account</code> — fund new accounts with starting XLM</li>
          <li><code className="text-green-500 text-xs">change_trust</code> — add USDC trustline during registration</li>
          <li><code className="text-green-500 text-xs">path_payment_strict_send</code> — DEX swaps with path finding</li>
          <li><code className="text-green-500 text-xs">fee_bump</code> — platform pays all fees</li>
        </ul>
      </section>

      {/* Anchor Integration */}
      <section>
        <h2 className="text-2xl font-bold mb-4">Anchor Integration (SEP-24)</h2>
        <p className="text-sm text-zinc-400 mb-4">
          Deposits and withdrawals use the Stellar SEP-24 protocol with any compliant anchor.
        </p>

        <h3 className="text-lg font-semibold mb-3">Order Lifecycle</h3>
        <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4 mb-4">
          <pre className="font-mono text-sm text-zinc-400">
{`pending → processing → completed
                    → error
                    → expired`}
          </pre>
        </div>
        <p className="text-sm text-zinc-400">
          Once an order reaches a terminal state, its status is immutable. This prevents duplicate webhook callbacks from double-crediting users.
        </p>
      </section>
    </div>
  );
}
