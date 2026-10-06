"use client";

import { motion } from "framer-motion";

export function Architecture() {
  return (
    <section className="py-24 border-t border-zinc-800/60">
      <div className="mx-auto max-w-6xl px-6">
        <motion.div
          initial={{ opacity: 0, y: 16 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-100px" }}
          transition={{ duration: 0.5 }}
          className="text-center mb-16"
        >
          <h2 className="text-3xl sm:text-4xl font-bold tracking-tight">
            Architecture
          </h2>
          <p className="mt-4 text-zinc-400 max-w-2xl mx-auto">
            From USSD dial to Stellar settlement in milliseconds. Every layer
            is designed for the constraints of mobile money in Africa.
          </p>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-50px" }}
          transition={{ duration: 0.5 }}
          className="rounded-xl border border-zinc-800/60 bg-zinc-950/50 p-4 sm:p-8 overflow-x-auto glow-green"
        >
          <pre className="font-mono text-[10px] sm:text-sm text-zinc-400 leading-relaxed">
            {`User dials *384# on any phone
        │
  Telco USSD Gateway
        │
  Africa's Talking (POST /ussd/callback)
        │
  ┌─────▼──────┐
  │  actix-web  │──── IP allowlist + rate limiting (Redis)
  └─────┬──────┘
        │
  USSD State Machine (Redis session + Postgres user lookup)
        │
  ┌─────┼──────────────────┐
  │     │                  │
Stellar     Postgres        External APIs
─────────   ────────        ─────────────
TX builder  users           SEP-24 anchors
transfers   transactions    CoinGecko (rates)
swaps       anchor_orders   Airbills (bills)
trustlines  user_activity   Africa's Talking (SMS)
fee bumps   merchants`}
          </pre>
        </motion.div>

        {/* Design principles */}
        <div className="mt-16 grid sm:grid-cols-2 gap-6">
          {[
            {
              title: "USSD latency is sacred",
              desc: "Sessions timeout after ~30 seconds. Every handler responds fast. Heavy operations are fire-and-forget via tokio::spawn.",
            },
            {
              title: "Config over code",
              desc: "Deploy to a new country by changing env vars. Currency, anchor, and shortcode are all configurable.",
            },
            {
              title: "Atomic state transitions",
              desc: "Once an order reaches a terminal state (completed/error/expired), it cannot be changed. No double-credits.",
            },
            {
              title: "Fail safe",
              desc: "If a bill payment API fails after debiting USDC, the system auto-refunds. Money is never silently lost.",
            },
          ].map(({ title, desc }, i) => (
            <motion.div
              key={title}
              initial={{ opacity: 0, y: 16 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.4, delay: i * 0.08 }}
              className="p-5 rounded-lg border border-zinc-800/60 bg-zinc-900/20"
            >
              <h3 className="font-semibold text-white mb-2">{title}</h3>
              <p className="text-sm text-zinc-400 leading-relaxed">{desc}</p>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
