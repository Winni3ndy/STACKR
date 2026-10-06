"use client";

import Link from "next/link";
import { motion } from "framer-motion";
import { UssdSimulator } from "./ussd-simulator";

export function Hero() {
  return (
    <section className="relative pt-24 sm:pt-32 pb-16 sm:pb-24 overflow-hidden hero-grid">
      {/* Subtle radial glow */}
      <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[800px] h-[600px] bg-green-500/[0.04] rounded-full blur-3xl pointer-events-none" />

      <div className="mx-auto max-w-6xl px-6 relative">
        <div className="flex flex-col lg:flex-row items-center gap-12 lg:gap-16">
          {/* Left: copy */}
          <div className="flex-1 max-w-xl">
            <motion.div
              initial={{ opacity: 0, y: 16 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5 }}
              className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-zinc-800 bg-zinc-900/60 text-xs text-zinc-400 mb-6"
            >
              <span className="w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse" />
              Built on Stellar &middot; Open beta
            </motion.div>

            <motion.h1
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5, delay: 0.1 }}
              className="text-3xl sm:text-5xl lg:text-6xl font-bold tracking-tight leading-[1.1]"
            >
              Send money from
              <br />
              <span className="text-green-500">any phone</span> in
              <br />
              Africa
            </motion.h1>

            <motion.p
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5, delay: 0.2 }}
              className="mt-6 text-base sm:text-lg text-zinc-400 leading-relaxed max-w-lg"
            >
              Stackr turns any mobile phone into a stablecoin wallet. Your users
              dial a USSD code to send money, pay bills, and cash out &mdash; no
              app download, no internet, no smartphone required.
            </motion.p>

            <motion.p
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5, delay: 0.25 }}
              className="mt-3 text-sm text-zinc-500"
            >
              One REST API. You integrate it. We handle Stellar, wallets, and settlement.
            </motion.p>

            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5, delay: 0.3 }}
              className="mt-8 flex flex-wrap gap-4"
            >
              <Link
                href="/signup"
                className="inline-flex items-center gap-2 px-6 py-3 bg-green-500 hover:bg-green-600 text-black font-semibold rounded-lg transition-colors"
              >
                Get API Key &mdash; Free
                <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                </svg>
              </Link>
              <Link
                href="/docs"
                className="inline-flex items-center gap-2 px-6 py-3 border border-zinc-700 hover:border-zinc-500 text-zinc-300 hover:text-white rounded-lg transition-colors"
              >
                Read the Docs
              </Link>
            </motion.div>
          </div>

          {/* Right: simulator */}
          <motion.div
            initial={{ opacity: 0, x: 30 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.6, delay: 0.3 }}
            className="shrink-0 w-full max-w-[300px] sm:max-w-[340px] lg:w-auto lg:max-w-none mx-auto"
          >
            <UssdSimulator compact />
          </motion.div>
        </div>
      </div>
    </section>
  );
}
