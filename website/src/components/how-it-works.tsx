"use client";

import { motion } from "framer-motion";

const steps = [
  {
    num: "01",
    title: "User dials a USSD code",
    description:
      "No app needed. The user dials *384# from any phone — feature phone, smartphone, 2G or 4G. Works offline.",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M3 5a2 2 0 012-2h3.28a1 1 0 01.948.684l1.498 4.493a1 1 0 01-.502 1.21l-2.257 1.13a11.042 11.042 0 005.516 5.516l1.13-2.257a1 1 0 011.21-.502l4.493 1.498a1 1 0 01.684.949V19a2 2 0 01-2 2h-1C9.716 21 3 14.284 3 6V5z" />
      </svg>
    ),
  },
  {
    num: "02",
    title: "Stackr handles everything",
    description:
      "Wallet creation, Stellar transactions, currency conversion, fee bumping, and settlement — all in under 5 seconds.",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M13 10V3L4 14h7v7l9-11h-7z" />
      </svg>
    ),
  },
  {
    num: "03",
    title: "Money arrives instantly",
    description:
      "The recipient gets a confirmation SMS. Funds settle on Stellar in USDC. Cash out at any supported anchor.",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
    ),
  },
];

export function HowItWorks() {
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
            How it works
          </h2>
          <p className="mt-4 text-zinc-400 max-w-2xl mx-auto">
            From dial to settlement in three steps. No app downloads,
            no blockchain knowledge, no internet connection required.
          </p>
        </motion.div>

        <div className="grid md:grid-cols-3 gap-6 sm:gap-8">
          {steps.map(({ num, title, description, icon }, i) => (
            <motion.div
              key={num}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.4, delay: i * 0.1 }}
              className="relative p-6 rounded-xl border border-zinc-800/60 bg-zinc-900/20"
            >
              <div className="flex items-center gap-4 mb-4">
                <div className="w-10 h-10 rounded-lg bg-green-500/10 text-green-500 flex items-center justify-center shrink-0">
                  {icon}
                </div>
                <span className="text-xs font-mono text-zinc-600">{num}</span>
              </div>
              <h3 className="text-lg font-semibold text-white mb-2">
                {title}
              </h3>
              <p className="text-sm text-zinc-400 leading-relaxed">
                {description}
              </p>

              {/* Connector line (hidden on last item and mobile) */}
              {i < steps.length - 1 && (
                <div className="hidden md:block absolute top-1/2 -right-4 sm:-right-5 w-6 sm:w-8 border-t border-dashed border-zinc-700" />
              )}
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
