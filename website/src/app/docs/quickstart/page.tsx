import type { Metadata } from "next";
import Link from "next/link";

export const metadata: Metadata = {
  title: "Quickstart",
};

export default function QuickstartPage() {
  return (
    <div className="max-w-none">
      <h1 className="text-3xl font-bold tracking-tight mb-2">Quickstart</h1>
      <p className="text-zinc-400 text-lg mb-10">
        Get your API key and make your first request in three steps.
      </p>

      <div className="space-y-8">
        {/* Step 1 */}
        <div className="rounded-xl border border-zinc-800/60 bg-zinc-900/20 p-6">
          <div className="flex items-center gap-3 mb-4">
            <span className="w-8 h-8 rounded-full bg-green-500/10 border border-green-500/30 text-green-500 text-sm font-bold flex items-center justify-center">1</span>
            <h2 className="text-xl font-semibold text-white">Create your account</h2>
          </div>
          <p className="text-sm text-zinc-400 mb-4 leading-relaxed">
            Sign up on the Stackr dashboard to get your API key. No credit card required for the free tier.
          </p>
          <div className="rounded-lg border border-green-500/20 bg-green-500/[0.03] p-4">
            <p className="text-sm text-zinc-300 mb-2">Your API keys will look like this:</p>
            <div className="font-mono text-xs space-y-1">
              <p className="text-zinc-500"># Test key (testnet) — safe for development</p>
              <p className="text-zinc-300">sk_test_a1b2c3d4e5f6g7h8i9j0...</p>
              <p className="text-zinc-500 mt-2"># Live key (mainnet) — use in production</p>
              <p className="text-zinc-300">sk_live_a1b2c3d4e5f6g7h8i9j0...</p>
            </div>
          </div>
          <div className="rounded-lg border border-amber-900/50 bg-amber-950/20 p-3 mt-4">
            <p className="text-xs text-amber-400">
              <strong>Keep your secret keys safe.</strong> Never expose them in client-side code, public repos, or browser requests. Always call the Stackr API from your backend.
            </p>
          </div>
        </div>

        {/* Step 2 */}
        <div className="rounded-xl border border-zinc-800/60 bg-zinc-900/20 p-6">
          <div className="flex items-center gap-3 mb-4">
            <span className="w-8 h-8 rounded-full bg-green-500/10 border border-green-500/30 text-green-500 text-sm font-bold flex items-center justify-center">2</span>
            <h2 className="text-xl font-semibold text-white">Make your first API call</h2>
          </div>
          <p className="text-sm text-zinc-400 mb-4 leading-relaxed">
            Check that your key works by hitting the health endpoint, then try checking a balance:
          </p>
          <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4 space-y-6">
            <div>
              <p className="text-xs text-zinc-500 mb-2"># Verify your connection</p>
              <pre className="font-mono text-sm text-zinc-300 leading-relaxed">
{`curl https://api.usestackr.xyz/health
# {"status":"healthy","database":true,"redis":true,"stellar":true}`}
              </pre>
            </div>
            <div>
              <p className="text-xs text-zinc-500 mb-2"># Check a wallet balance</p>
              <pre className="font-mono text-sm text-zinc-300 leading-relaxed">
{`curl https://api.usestackr.xyz/api/v1/accounts/+250781234567/balance \\
  -H "Authorization: Bearer sk_test_your_key_here"`}
              </pre>
            </div>
            <div>
              <p className="text-xs text-zinc-500 mb-2"># Send USDC</p>
              <pre className="font-mono text-sm text-zinc-300 leading-relaxed">
{`curl -X POST https://api.usestackr.xyz/api/v1/transfers \\
  -H "Authorization: Bearer sk_test_your_key_here" \\
  -H "Content-Type: application/json" \\
  -d '{
    "sender_phone": "+250781234567",
    "recipient_phone": "+250789876543",
    "amount": "10.00",
    "pin": "1234"
  }'`}
              </pre>
            </div>
          </div>
        </div>

        {/* Step 3 */}
        <div className="rounded-xl border border-zinc-800/60 bg-zinc-900/20 p-6">
          <div className="flex items-center gap-3 mb-4">
            <span className="w-8 h-8 rounded-full bg-green-500/10 border border-green-500/30 text-green-500 text-sm font-bold flex items-center justify-center">3</span>
            <h2 className="text-xl font-semibold text-white">Integrate into your app</h2>
          </div>
          <p className="text-sm text-zinc-400 mb-4 leading-relaxed">
            Call the Stackr API from your backend. Here are examples in common languages:
          </p>

          <div className="space-y-4">
            {/* Node.js example */}
            <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4">
              <p className="text-xs text-zinc-500 mb-2">Node.js / JavaScript</p>
              <pre className="font-mono text-xs text-zinc-300 leading-relaxed">
{`const response = await fetch("https://api.usestackr.xyz/api/v1/transfers", {
  method: "POST",
  headers: {
    "Authorization": \`Bearer \${process.env.STACKR_API_KEY}\`,
    "Content-Type": "application/json",
  },
  body: JSON.stringify({
    sender_phone: "+250781234567",
    recipient_phone: "+250789876543",
    amount: "10.00",
    pin: "1234",
  }),
});

const result = await response.json();
console.log(result.tx_hash); // "3d8f4a2b..."
`}
              </pre>
            </div>

            {/* Python example */}
            <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4">
              <p className="text-xs text-zinc-500 mb-2">Python</p>
              <pre className="font-mono text-xs text-zinc-300 leading-relaxed">
{`import requests
import os

response = requests.post(
    "https://api.usestackr.xyz/api/v1/transfers",
    headers={"Authorization": f"Bearer {os.environ['STACKR_API_KEY']}"},
    json={
        "sender_phone": "+250781234567",
        "recipient_phone": "+250789876543",
        "amount": "10.00",
        "pin": "1234",
    },
)

result = response.json()
print(result["tx_hash"])  # "3d8f4a2b..."
`}
              </pre>
            </div>
          </div>
        </div>
      </div>

      {/* What's next */}
      <div className="mt-10 p-5 rounded-xl border border-zinc-800/60 bg-zinc-900/20">
        <h3 className="font-semibold text-white mb-3">Next steps</h3>
        <ul className="space-y-2 text-sm text-zinc-400">
          <li className="flex items-start gap-2">
            <span className="text-green-500">&#8226;</span>
            Browse the <Link href="/docs/api" className="text-green-500 hover:underline">API Reference</Link> for all 17 endpoints with request/response examples
          </li>
          <li className="flex items-start gap-2">
            <span className="text-green-500">&#8226;</span>
            Read the <Link href="/docs/guide" className="text-green-500 hover:underline">Developer Guide</Link> to understand the USSD protocol, wallet system, and Stellar integration
          </li>
          <li className="flex items-start gap-2">
            <span className="text-green-500">&#8226;</span>
            Review <Link href="/docs/security" className="text-green-500 hover:underline">Security</Link> best practices for handling API keys and user data
          </li>
          <li className="flex items-start gap-2">
            <span className="text-green-500">&#8226;</span>
            Check out <Link href="/pricing" className="text-green-500 hover:underline">Pricing</Link> to see which plan fits your usage
          </li>
        </ul>
      </div>
    </div>
  );
}
