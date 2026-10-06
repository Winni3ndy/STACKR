"use client";

import { motion } from "framer-motion";
import Link from "next/link";

const tiers = [
  {
    name: "Free",
    price: "$0",
    period: "forever",
    description: "For testing and building your integration.",
    cta: "Get Started Free",
    ctaHref: "/signup",
    highlight: false,
    features: [
      "1,000 API calls / month",
      "Testnet access",
      "All 17 endpoints",
      "Community support",
      "Dashboard & usage analytics",
    ],
  },
  {
    name: "Pro",
    price: "$49",
    period: "/ month",
    description: "For apps in production with real users.",
    cta: "Start Pro Trial",
    ctaHref: "/signup?plan=pro",
    highlight: true,
    features: [
      "50,000 API calls / month",
      "Mainnet access",
      "All 17 endpoints",
      "Priority email support",
      "Dashboard & usage analytics",
      "Webhook notifications",
      "Higher rate limits",
      "$0.002 per call after quota",
    ],
  },
  {
    name: "Enterprise",
    price: "Custom",
    period: "",
    description: "For high-volume operators and telcos.",
    cta: "Contact Sales",
    ctaHref: "/contact",
    highlight: false,
    features: [
      "Unlimited API calls",
      "Mainnet access",
      "Dedicated infrastructure",
      "24/7 support + SLA",
      "Custom rate limits",
      "Webhook notifications",
      "Volume discounts",
      "White-label options",
    ],
  },
];

export default function PricingPage() {
  return (
    <main className="pt-32 pb-24">
      <div className="mx-auto max-w-6xl px-6">
        <motion.div
          initial={{ opacity: 0, y: 16 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5 }}
          className="text-center mb-16"
        >
          <h1 className="text-4xl sm:text-5xl font-bold tracking-tight">
            Simple, transparent pricing
          </h1>
          <p className="mt-4 text-lg text-zinc-400 max-w-2xl mx-auto">
            Start free. Scale as you grow. No hidden fees, no surprise charges.
            Pay only for what you use.
          </p>
        </motion.div>

        <div className="grid md:grid-cols-3 gap-6 max-w-5xl mx-auto">
          {tiers.map((tier, i) => (
            <motion.div
              key={tier.name}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5, delay: i * 0.1 }}
              className={`relative rounded-xl border p-5 sm:p-8 flex flex-col ${
                tier.highlight
                  ? "border-green-500/50 bg-green-500/[0.03]"
                  : "border-zinc-800/60 bg-zinc-900/20"
              }`}
            >
              {tier.highlight && (
                <div className="absolute -top-3 left-1/2 -translate-x-1/2">
                  <span className="px-3 py-1 bg-green-500 text-black text-xs font-semibold rounded-full">
                    Most Popular
                  </span>
                </div>
              )}

              <div className="mb-6">
                <h3 className="text-lg font-semibold text-white">{tier.name}</h3>
                <div className="mt-3 flex items-baseline gap-1">
                  <span className="text-4xl font-bold text-white">{tier.price}</span>
                  {tier.period && (
                    <span className="text-sm text-zinc-500">{tier.period}</span>
                  )}
                </div>
                <p className="mt-2 text-sm text-zinc-400">{tier.description}</p>
              </div>

              <ul className="space-y-3 mb-8 flex-1">
                {tier.features.map((feature) => (
                  <li key={feature} className="flex items-start gap-2 text-sm text-zinc-300">
                    <svg className="w-4 h-4 text-green-500 shrink-0 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                    </svg>
                    {feature}
                  </li>
                ))}
              </ul>

              <Link
                href={tier.ctaHref}
                className={`block text-center py-3 px-4 rounded-lg text-sm font-semibold transition-colors ${
                  tier.highlight
                    ? "bg-green-500 hover:bg-green-600 text-black"
                    : "border border-zinc-700 hover:border-zinc-500 text-zinc-300 hover:text-white"
                }`}
              >
                {tier.cta}
              </Link>
            </motion.div>
          ))}
        </div>

        {/* FAQ-style details */}
        <motion.div
          initial={{ opacity: 0, y: 16 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-20 max-w-3xl mx-auto"
        >
          <h2 className="text-2xl font-bold text-center mb-10">
            Common questions
          </h2>
          <div className="space-y-6">
            {[
              {
                q: "What counts as an API call?",
                a: "Every request to a Stackr API endpoint counts as one call. Health checks and webhook deliveries from Stackr to your server do not count.",
              },
              {
                q: "What happens if I exceed my quota?",
                a: "On the Free tier, additional calls return 429 until the next billing cycle. On Pro, overage is billed at $0.002 per call. Enterprise plans have custom arrangements.",
              },
              {
                q: "Can I switch plans anytime?",
                a: "Yes. Upgrade or downgrade at any time from your dashboard. Changes take effect immediately, with prorated billing.",
              },
              {
                q: "Is there a transaction fee on top of the API fee?",
                a: "Stackr does not charge a separate transaction fee. The Stellar network fee (~0.00001 XLM) is included in all operations.",
              },
              {
                q: "Do you support testnet for free?",
                a: "Yes. Both Free and Pro plans have full testnet access. We recommend building and testing on testnet before going to mainnet.",
              },
            ].map(({ q, a }) => (
              <div key={q} className="rounded-xl border border-zinc-800/60 bg-zinc-900/20 p-5">
                <h3 className="font-semibold text-white text-sm">{q}</h3>
                <p className="mt-2 text-sm text-zinc-400 leading-relaxed">{a}</p>
              </div>
            ))}
          </div>
        </motion.div>
      </div>
    </main>
  );
}
