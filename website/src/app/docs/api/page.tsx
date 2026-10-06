import type { Metadata } from "next";
import { endpoints } from "@/lib/api-data";
import { ApiEndpointCard } from "@/components/api-endpoint";

export const metadata: Metadata = {
  title: "API Reference",
};

export default function ApiReferencePage() {
  const coreEndpoints = endpoints.filter((e) => !e.path.startsWith("/api/v1"));
  const apiEndpoints = endpoints.filter((e) => e.path.startsWith("/api/v1"));

  return (
    <div className="max-w-none">
      <h1 className="text-3xl font-bold tracking-tight mb-2">API Reference</h1>
      <p className="text-zinc-400 text-lg mb-8">
        17 endpoints with curl examples, request/response payloads, and error codes.
      </p>

      {/* Base URL */}
      <div className="rounded-xl border border-zinc-800/60 bg-zinc-900/20 p-5 mb-6">
        <h3 className="text-sm font-semibold text-white mb-2">Base URL</h3>
        <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-3">
          <pre className="font-mono text-sm text-zinc-300">https://api.usestackr.xyz</pre>
        </div>
        <p className="text-xs text-zinc-500 mt-2">
          Testnet: <code className="text-zinc-400">https://testnet.api.usestackr.xyz</code>
        </p>
      </div>

      {/* Auth section */}
      <div className="rounded-xl border border-zinc-800/60 bg-zinc-900/20 p-5 mb-6">
        <h3 className="text-sm font-semibold text-white mb-3">Authentication</h3>
        <p className="text-sm text-zinc-400 mb-4">
          API v1 endpoints require your secret API key in the <code className="text-zinc-300 bg-zinc-800 px-1.5 py-0.5 rounded text-xs">Authorization</code> header.
          Get your API key from the <strong className="text-zinc-300">Stackr Dashboard</strong> after signing up.
        </p>

        <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-3 mb-4">
          <pre className="font-mono text-xs text-zinc-400">Authorization: Bearer sk_live_your_api_key_here</pre>
        </div>

        <div className="grid sm:grid-cols-2 gap-3">
          <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-3">
            <p className="text-xs text-zinc-500 mb-1">Test key (testnet)</p>
            <p className="font-mono text-xs text-zinc-300">sk_test_...</p>
            <p className="text-[10px] text-zinc-600 mt-1">Safe for development. No real money moves.</p>
          </div>
          <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-3">
            <p className="text-xs text-zinc-500 mb-1">Live key (mainnet)</p>
            <p className="font-mono text-xs text-zinc-300">sk_live_...</p>
            <p className="text-[10px] text-zinc-600 mt-1">Production. Real USDC on Stellar mainnet.</p>
          </div>
        </div>

        <div className="rounded-lg border border-amber-900/50 bg-amber-950/20 p-3 mt-4">
          <p className="text-xs text-amber-400">
            <strong>Never expose your secret key in client-side code.</strong> Always make Stackr API calls from your backend server.
          </p>
        </div>
      </div>

      {/* Rate limiting */}
      <div className="rounded-xl border border-zinc-800/60 bg-zinc-900/20 p-5 mb-10">
        <h3 className="text-sm font-semibold text-white mb-2">Rate Limiting</h3>
        <p className="text-sm text-zinc-400 mb-3">
          Rate limits are per API key, not per IP. Limits depend on your plan:
        </p>
        <div className="grid sm:grid-cols-3 gap-3">
          {[
            { plan: "Free", limit: "60 req/min", calls: "1,000/month" },
            { plan: "Pro", limit: "300 req/min", calls: "50,000/month" },
            { plan: "Enterprise", limit: "Custom", calls: "Unlimited" },
          ].map(({ plan, limit, calls }) => (
            <div key={plan} className="rounded-lg border border-zinc-800 bg-zinc-950 p-3 text-center">
              <p className="text-xs text-zinc-500">{plan}</p>
              <p className="font-mono text-sm text-zinc-300 mt-1">{limit}</p>
              <p className="text-[10px] text-zinc-600 mt-0.5">{calls}</p>
            </div>
          ))}
        </div>
        <p className="text-sm text-zinc-400 mt-3">
          When rate-limited, the server returns <code className="text-red-400/80 bg-zinc-800 px-1.5 py-0.5 rounded text-xs">429 Too Many Requests</code>.
        </p>
      </div>

      {/* Core endpoints */}
      <h2 className="text-xl font-bold mb-4 flex items-center gap-2">
        Core Endpoints
        <span className="text-xs text-zinc-600 font-normal">No authentication required</span>
      </h2>
      <div className="space-y-3 mb-12">
        {coreEndpoints.map((ep) => (
          <ApiEndpointCard key={ep.path + ep.method} endpoint={ep} />
        ))}
      </div>

      {/* API v1 */}
      <h2 className="text-xl font-bold mb-4 flex items-center gap-2">
        API v1
        <span className="text-xs text-amber-500/80 border border-amber-500/20 bg-amber-500/5 px-1.5 py-0.5 rounded font-medium">
          AUTH REQUIRED
        </span>
      </h2>
      <div className="space-y-3 mb-12">
        {apiEndpoints.map((ep) => (
          <ApiEndpointCard key={ep.path + ep.method} endpoint={ep} />
        ))}
      </div>

      {/* Error format */}
      <h2 className="text-xl font-bold mb-4">Error Format</h2>
      <p className="text-sm text-zinc-400 mb-4">
        All errors follow a consistent JSON structure:
      </p>
      <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4 mb-12">
        <pre className="font-mono text-xs text-zinc-300 leading-relaxed">
{`{
  "error": "Insufficient balance",
  "code": "INSUFFICIENT_BALANCE",
  "details": "Account has 5.00 USDC, transfer requires 10.00 USDC"
}`}
        </pre>
      </div>
    </div>
  );
}
