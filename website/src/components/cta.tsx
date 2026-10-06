"use client";

import { motion } from "framer-motion";
import Link from "next/link";

export function CTA() {
  return (
    <section className="py-24 border-t border-zinc-800/60 relative overflow-hidden">
      <div className="absolute bottom-0 left-1/2 -translate-x-1/2 w-[600px] h-[400px] bg-green-500/[0.03] rounded-full blur-3xl pointer-events-none" />

      <div className="mx-auto max-w-6xl px-6 text-center relative">
        <motion.div
          initial={{ opacity: 0, y: 16 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
        >
          <h2 className="text-3xl sm:text-4xl font-bold tracking-tight">
            Ready to give your users
            <br className="hidden sm:block" />
            stablecoin payments?
          </h2>
          <p className="mt-4 text-zinc-400 max-w-xl mx-auto">
            Get your API key in seconds. The free tier includes 1,000 calls
            per month &mdash; enough to build and test a full integration.
          </p>

          <div className="mt-8 rounded-xl border border-zinc-800/60 bg-zinc-950/50 p-4 max-w-lg mx-auto">
            <code className="font-mono text-sm text-zinc-300">
              curl https://api.usestackr.xyz/health
            </code>
            <p className="text-xs text-zinc-600 mt-2">Try it now &mdash; no API key needed</p>
          </div>

          <div className="mt-8 flex justify-center gap-4 flex-wrap">
            <Link
              href="/signup"
              className="inline-flex items-center gap-2 px-6 py-3 bg-green-500 hover:bg-green-600 text-black font-semibold rounded-lg transition-colors"
            >
              Get Your API Key
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
              </svg>
            </Link>
            <Link
              href="/pricing"
              className="inline-flex items-center gap-2 px-6 py-3 border border-zinc-700 hover:border-zinc-500 text-zinc-300 hover:text-white rounded-lg transition-colors"
            >
              View Pricing
            </Link>
          </div>
        </motion.div>
      </div>
    </section>
  );
}
