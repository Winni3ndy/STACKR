"use client";

import { useState } from "react";
import { motion } from "framer-motion";

const tabs = [
  {
    label: "Send Money",
    code: `# Transfer USDC between phone numbers
curl -X POST https://api.usestackr.xyz/api/v1/transfers \\
  -H "Authorization: Bearer sk_live_abc123..." \\
  -H "Content-Type: application/json" \\
  -d '{
    "sender_phone": "+250781234567",
    "recipient_phone": "+250789876543",
    "amount": "10.00",
    "pin": "1234"
  }'

# Response
{
  "tx_hash": "3d8f4a2b1c9e...",
  "status": "completed",
  "fee": "0.00001"
}`,
  },
  {
    label: "Check Balance",
    code: `# Get wallet balance for a phone number
curl https://api.usestackr.xyz/api/v1/balance/+250781234567 \\
  -H "Authorization: Bearer sk_live_abc123..."

# Response
{
  "phone": "+250781234567",
  "balances": {
    "USDC": "142.50",
    "XLM": "15.0000000"
  },
  "public_key": "GCXYZ..."
}`,
  },
  {
    label: "Pay Bills",
    code: `# Buy airtime for a phone number
curl -X POST https://api.usestackr.xyz/api/v1/airtime \\
  -H "Authorization: Bearer sk_live_abc123..." \\
  -H "Content-Type: application/json" \\
  -d '{
    "phone": "+250781234567",
    "amount": "5.00",
    "pin": "1234"
  }'

# Response
{
  "status": "delivered",
  "local_amount": "5000 RWF",
  "usdc_charged": "3.85"
}`,
  },
  {
    label: "USSD Callback",
    code: `# Your USSD provider sends requests here
# Stackr handles the entire session state machine

POST /ussd/callback
Content-Type: application/x-www-form-urlencoded

sessionId=ATSid_123&phoneNumber=+250781234567
&text=1*+250789876543*10*1234
&serviceCode=*384#

# Stackr responds with USSD menu text
CON Transfer 10.00 USDC to +250789876543?
1. Confirm
2. Cancel`,
  },
];

export function CodePreview() {
  const [active, setActive] = useState(0);

  return (
    <section className="py-24 border-t border-zinc-800/60">
      <div className="mx-auto max-w-6xl px-6">
        <motion.div
          initial={{ opacity: 0, y: 16 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-100px" }}
          transition={{ duration: 0.5 }}
          className="text-center mb-12"
        >
          <h2 className="text-3xl sm:text-4xl font-bold tracking-tight">
            Integrate in minutes
          </h2>
          <p className="mt-4 text-zinc-400 max-w-2xl mx-auto">
            Simple RESTful API. Get your API key, make a request, and you have
            stablecoin payments working. No blockchain knowledge required.
          </p>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-50px" }}
          transition={{ duration: 0.5 }}
          className="rounded-xl border border-zinc-800/60 bg-zinc-950/50 overflow-hidden"
        >
          {/* Tabs */}
          <div className="flex border-b border-zinc-800/60 overflow-x-auto">
            {tabs.map((tab, i) => (
              <button
                key={tab.label}
                onClick={() => setActive(i)}
                className={`px-3 sm:px-5 py-3 sm:py-3.5 text-xs sm:text-sm font-medium whitespace-nowrap transition-colors ${
                  i === active
                    ? "text-green-500 border-b-2 border-green-500 bg-zinc-900/30"
                    : "text-zinc-500 hover:text-zinc-300"
                }`}
              >
                {tab.label}
              </button>
            ))}
          </div>

          {/* Code */}
          <pre className="p-3 sm:p-6 overflow-x-auto text-[11px] sm:text-sm leading-relaxed min-h-[250px] sm:min-h-[300px]">
            <code className="font-mono text-zinc-300">
              {tabs[active].code}
            </code>
          </pre>
        </motion.div>
      </div>
    </section>
  );
}
