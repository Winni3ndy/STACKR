"use client";

import { motion } from "framer-motion";

const features = [
  {
    title: "Send Money by Phone Number",
    description:
      "Users send stablecoins to any phone number. One API call handles wallet lookup, Stellar transactions, and fee bumping automatically.",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
      </svg>
    ),
  },
  {
    title: "Pay Bills with USDC",
    description:
      "Airtime, data, electricity, cable TV. Users pay in stablecoins, Stackr converts and delivers to local providers instantly.",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 14l6-6m-5.5.5h.01m4.99 5h.01M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16l3.5-2 3.5 2 3.5-2 3.5 2z" />
      </svg>
    ),
  },
  {
    title: "Cash In / Cash Out",
    description:
      "Deposit local currency to get USDC. Withdraw USDC to receive mobile money. Integrated with SEP-24 anchors for compliant fiat rails.",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
    ),
  },
  {
    title: "Merchant Payments",
    description:
      "Issue merchant codes via API. Customers pay by entering the code on USSD. Instant USDC settlement, no POS hardware needed.",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M3 3h2l.4 2M7 13h10l4-8H5.4M7 13L5.4 5M7 13l-2.293 2.293c-.63.63-.184 1.707.707 1.707H17m0 0a2 2 0 100 4 2 2 0 000-4zm-8 2a2 2 0 100 4 2 2 0 000-4z" />
      </svg>
    ),
  },
  {
    title: "Token Swaps",
    description:
      "USDC/XLM swaps on the Stellar DEX, exposed as a simple API call. Real-time pricing, automatic path finding, slippage protection.",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
      </svg>
    ),
  },
  {
    title: "Auto-Managed Wallets",
    description:
      "Every phone number gets a Stellar wallet automatically. No seed phrases, no key management for end users. Fully custodial.",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
      </svg>
    ),
  },
];

const containerVariants = {
  hidden: {},
  visible: {
    transition: {
      staggerChildren: 0.08,
    },
  },
};

const itemVariants = {
  hidden: { opacity: 0, y: 20 },
  visible: { opacity: 1, y: 0, transition: { duration: 0.4 } },
};

export function FeatureGrid() {
  return (
    <section className="py-24">
      <div className="mx-auto max-w-6xl px-6">
        <motion.div
          initial={{ opacity: 0, y: 16 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-100px" }}
          transition={{ duration: 0.5 }}
          className="text-center mb-16"
        >
          <h2 className="text-3xl sm:text-4xl font-bold tracking-tight">
            Everything your users need
          </h2>
          <p className="mt-4 text-zinc-400 max-w-2xl mx-auto">
            A complete financial toolkit accessible from any phone.
            Integrate once and give your users access to all of these.
          </p>
        </motion.div>

        <motion.div
          variants={containerVariants}
          initial="hidden"
          whileInView="visible"
          viewport={{ once: true, margin: "-50px" }}
          className="grid sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6"
        >
          {features.map(({ title, description, icon }) => (
            <motion.div
              key={title}
              variants={itemVariants}
              className="group p-4 sm:p-6 rounded-xl border border-zinc-800/60 bg-zinc-900/20 hover:border-zinc-700 hover:bg-zinc-900/50 transition-all"
            >
              <div className="w-10 h-10 rounded-lg bg-green-500/10 text-green-500 flex items-center justify-center mb-4 group-hover:bg-green-500/20 transition-colors">
                {icon}
              </div>
              <h3 className="text-lg font-semibold text-white mb-2">
                {title}
              </h3>
              <p className="text-sm text-zinc-400 leading-relaxed">
                {description}
              </p>
            </motion.div>
          ))}
        </motion.div>
      </div>
    </section>
  );
}
