"use client";

import { motion } from "framer-motion";

const stats = [
  { label: "Endpoints", value: "17", sub: "Full REST API" },
  { label: "Avg Response", value: "<100ms", sub: "USSD-grade latency" },
  { label: "Min Phone", value: "2G", sub: "No internet needed" },
  { label: "Settlement", value: "~5s", sub: "Stellar finality" },
];

export function StatsBar() {
  return (
    <section className="border-y border-zinc-800/60 bg-zinc-900/20">
      <div className="mx-auto max-w-6xl px-6 py-14">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 sm:gap-8">
          {stats.map(({ label, value, sub }, i) => (
            <motion.div
              key={label}
              initial={{ opacity: 0, y: 12 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.4, delay: i * 0.08 }}
              className="text-center"
            >
              <p className="text-2xl sm:text-3xl font-bold text-white">
                {value}
              </p>
              <p className="text-sm text-zinc-400 mt-1">{label}</p>
              <p className="text-xs text-zinc-600 mt-0.5">{sub}</p>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
