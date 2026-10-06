import type { Metadata } from "next";
import Link from "next/link";
import { FadeIn, StaggerContainer, StaggerItem } from "@/components/motion-wrapper";

export const metadata: Metadata = {
  title: "Documentation",
};

export default function DocsOverview() {
  return (
    <div className="max-w-none">
      <FadeIn>
        <h1 className="text-3xl font-bold tracking-tight mb-2">Documentation</h1>
        <p className="text-zinc-400 text-lg mb-10">
          Everything you need to integrate Stackr&apos;s stablecoin payment API into your application.
        </p>
      </FadeIn>

      {/* Quick links */}
      <StaggerContainer className="grid sm:grid-cols-2 gap-4 mb-16">
        {[
          {
            href: "/docs/quickstart",
            title: "Quickstart",
            desc: "Get your API key and make your first request in three steps.",
            badge: "Start here",
          },
          {
            href: "/docs/api",
            title: "API Reference",
            desc: "All 17 endpoints with curl examples, error codes, and payloads.",
            badge: null,
          },
          {
            href: "/docs/guide",
            title: "Developer Guide",
            desc: "USSD protocol, wallet system, Stellar integration, and how it all connects.",
            badge: null,
          },
          {
            href: "/docs/security",
            title: "Security",
            desc: "How we protect user data, keys, and funds. Best practices for your integration.",
            badge: null,
          },
        ].map(({ href, title, desc, badge }) => (
          <StaggerItem key={href}>
            <Link
              href={href}
              className="block p-5 rounded-lg border border-zinc-800/60 hover:border-zinc-700 bg-zinc-900/20 hover:bg-zinc-900/50 transition-all group relative h-full"
            >
              {badge && (
                <span className="absolute top-3 right-3 px-2 py-0.5 bg-green-500/10 border border-green-500/30 text-green-500 text-[10px] font-semibold rounded-full">
                  {badge}
                </span>
              )}
              <h2 className="text-lg font-semibold text-white group-hover:text-green-500 transition-colors">
                {title}
              </h2>
              <p className="text-sm text-zinc-400 mt-1">{desc}</p>
            </Link>
          </StaggerItem>
        ))}
      </StaggerContainer>

      {/* How Stackr works (brief overview for API consumers) */}
      <FadeIn>
        <h2 className="text-2xl font-bold mb-2">How Stackr works</h2>
        <p className="text-zinc-400 mb-8 text-sm leading-relaxed max-w-2xl">
          Stackr is a hosted API that handles stablecoin payments over USSD. You call our REST API from your backend — we handle wallet management, Stellar transactions, and USSD session state.
        </p>
      </FadeIn>

      <FadeIn>
        <div className="rounded-xl border border-zinc-800/60 bg-zinc-900/20 p-6 mb-10">
          <pre className="font-mono text-xs text-zinc-400 leading-loose">
{`Your App                     Stackr API                    Stellar Network
   │                             │                              │
   ├── POST /v1/transfers ──────>│                              │
   │   (sender, recipient, amt)  │── build + sign tx ──────────>│
   │                             │<── tx_hash ─────────────────│
   │<── { tx_hash, status } ─────│                              │
   │                             │                              │

   Your app never touches Stellar directly.
   We handle wallets, fees, signing, and broadcasting.`}
          </pre>
        </div>
      </FadeIn>

      {/* Key concepts */}
      <FadeIn>
        <h2 className="text-2xl font-bold mb-6">Key concepts</h2>
      </FadeIn>
      <StaggerContainer className="space-y-4 mb-12">
        {[
          {
            title: "API Keys",
            desc: "Authenticate every request with your secret key in the Authorization header. Get keys from your dashboard. Test keys (sk_test_...) work on testnet; live keys (sk_live_...) work on mainnet.",
          },
          {
            title: "Phone-based wallets",
            desc: "Every phone number automatically gets a Stellar wallet. No setup needed from your users. Wallets are created on first use and managed entirely by Stackr.",
          },
          {
            title: "Idempotency",
            desc: "All money-movement endpoints are idempotent. If a request times out, retry safely — duplicate transactions are impossible. The system uses Redis-backed idempotency keys internally.",
          },
          {
            title: "USSD integration",
            desc: "If you want end users to interact via USSD menus (dial *384#), point your Africa's Talking callback URL to Stackr. Otherwise, use the REST API directly from your app.",
          },
          {
            title: "Rate limits",
            desc: "API calls are rate-limited per key. Free tier: 60 req/min. Pro: 300 req/min. Enterprise: custom. Exceeding the limit returns 429 Too Many Requests.",
          },
        ].map(({ title, desc }) => (
          <StaggerItem key={title}>
            <div className="rounded-lg border border-zinc-800/60 bg-zinc-900/10 p-4">
              <h3 className="text-sm font-semibold text-white mb-1">{title}</h3>
              <p className="text-sm text-zinc-400 leading-relaxed">{desc}</p>
            </div>
          </StaggerItem>
        ))}
      </StaggerContainer>

      {/* SDKs / Languages */}
      <FadeIn>
        <h2 className="text-2xl font-bold mb-4">Works with any language</h2>
        <p className="text-zinc-400 text-sm mb-6">
          Stackr is a standard REST API. No SDK required — use <code className="text-zinc-300 bg-zinc-800 px-1.5 py-0.5 rounded text-xs">curl</code>, <code className="text-zinc-300 bg-zinc-800 px-1.5 py-0.5 rounded text-xs">fetch</code>, <code className="text-zinc-300 bg-zinc-800 px-1.5 py-0.5 rounded text-xs">requests</code>, or any HTTP client.
        </p>
      </FadeIn>
      <StaggerContainer className="grid grid-cols-3 sm:grid-cols-6 gap-3 mb-12">
        {["JavaScript", "Python", "Ruby", "Go", "PHP", "Rust"].map((lang) => (
          <StaggerItem key={lang}>
            <div className="text-center p-3 rounded-lg border border-zinc-800/60 bg-zinc-900/20">
              <span className="text-xs text-zinc-400 font-mono">{lang}</span>
            </div>
          </StaggerItem>
        ))}
      </StaggerContainer>

      {/* Next steps */}
      <FadeIn>
        <div className="p-5 rounded-xl border border-green-500/20 bg-green-500/[0.02]">
          <h3 className="font-semibold text-white mb-3">Ready to start?</h3>
          <p className="text-sm text-zinc-400 mb-4">
            Head to the <Link href="/docs/quickstart" className="text-green-500 hover:underline font-medium">Quickstart guide</Link> to get your API key and make your first request.
          </p>
          <Link
            href="/docs/quickstart"
            className="inline-flex items-center gap-2 px-5 py-2.5 bg-green-500 hover:bg-green-600 text-black text-sm font-semibold rounded-lg transition-colors"
          >
            Get Started
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
            </svg>
          </Link>
        </div>
      </FadeIn>
    </div>
  );
}
