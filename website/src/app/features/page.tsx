import type { Metadata } from "next";
import { FadeIn, StaggerContainer, StaggerItem } from "@/components/motion-wrapper";

export const metadata: Metadata = {
  title: "Features",
};

const features = [
  {
    title: "P2P USDC Transfers",
    description:
      "Send stablecoins to any phone number. Transactions settle on Stellar in seconds. Fee-bumped so users never need to hold or acquire XLM.",
    details: [
      "Recipient identified by phone number",
      "Idempotency keys prevent double-sends",
      "Fee payer covers all Stellar transaction fees",
      "Activity logged asynchronously (never blocks USSD)",
    ],
  },
  {
    title: "Bill Payments",
    description:
      "Pay utility bills, buy airtime and data — all from a USSD menu. USDC is debited, fiat is delivered to the provider via Airbills API.",
    details: [
      "Airtime, data, electricity, cable TV, internet, betting",
      "Auto-refund on provider API failure",
      "Fiat conversion via real-time CoinGecko rates",
      "Daily spend limits enforced per KYC tier",
    ],
  },
  {
    title: "Fiat On/Off Ramp (SEP-24)",
    description:
      "Deposit local currency to get USDC. Withdraw USDC to receive local currency. Uses any Stellar SEP-24 compliant anchor.",
    details: [
      "SEP-10 authentication with anchor",
      "Deposit: fiat → mobile money → anchor → USDC to wallet",
      "Withdrawal: USDC → anchor → mobile money → fiat",
      "Atomic order lifecycle: pending → processing → completed",
    ],
  },
  {
    title: "Merchant Payments",
    description:
      "Merchants register with a code. Users enter the merchant code and amount to pay instantly in USDC.",
    details: [
      "Merchant lookup by short code",
      "Instant USDC settlement to merchant wallet",
      "Transaction receipts via SMS",
      "Merchant table in Postgres with public key mapping",
    ],
  },
  {
    title: "Token Swaps",
    description:
      "Swap between USDC and XLM using Stellar's built-in DEX. Path payments find the best route automatically.",
    details: [
      "path_payment_strict_send for predictable input amounts",
      "Real-time XLM/USD pricing from CoinGecko",
      "Automatic path finding on Stellar DEX",
      "Fee-bumped like all other transactions",
    ],
  },
  {
    title: "Custodial Wallet System",
    description:
      "Deterministic key derivation from a master seed. Each phone number maps to exactly one Stellar keypair. No seed phrases, no key backup needed by users.",
    details: [
      "HKDF-SHA256 key derivation",
      "AES-256-GCM encryption at rest",
      "Argon2id PIN hashing",
      "Same seed + phone = same wallet, always",
    ],
  },
  {
    title: "Multi-Country Support",
    description:
      "Deploy to any African country by changing two environment variables: FIAT_CURRENCY and FIAT_COUNTRY.",
    details: [
      "RWF, NGN, KES, GHS, and any CoinGecko-supported currency",
      "Configurable USSD shortcode",
      "Swappable SEP-24 anchor via config",
      "Localized bill payment categories",
    ],
  },
  {
    title: "Security-First Design",
    description:
      "Built with Rust's memory safety, constant-time comparisons, and defense-in-depth security model.",
    details: [
      "IP allowlist for Africa's Talking callbacks",
      "Redis-backed rate limiting",
      "Constant-time webhook secret verification",
      "Idempotency keys on all money movements",
    ],
  },
];

export default function FeaturesPage() {
  return (
    <div className="pt-32 pb-20">
      <div className="mx-auto max-w-6xl px-6">
        <FadeIn className="text-center mb-16">
          <h1 className="text-4xl sm:text-5xl font-bold tracking-tight">
            Features
          </h1>
          <p className="mt-4 text-lg text-zinc-400 max-w-2xl mx-auto">
            A complete stablecoin payment stack for mobile money markets.
            Every feature accessible via USSD — no app required.
          </p>
        </FadeIn>

        <StaggerContainer className="space-y-8">
          {features.map(({ title, description, details }, i) => (
            <StaggerItem key={title}>
              <div className="p-6 sm:p-8 rounded-xl border border-zinc-800 bg-zinc-900/30 hover:border-zinc-700 transition-colors">
                <div className="flex items-start gap-4 mb-4">
                  <span className="text-xs text-zinc-600 font-mono mt-1">
                    {String(i + 1).padStart(2, "0")}
                  </span>
                  <div>
                    <h2 className="text-xl font-bold text-white">{title}</h2>
                    <p className="text-zinc-400 mt-2 leading-relaxed">
                      {description}
                    </p>
                    <ul className="mt-4 grid sm:grid-cols-2 gap-2">
                      {details.map((d) => (
                        <li
                          key={d}
                          className="flex items-start gap-2 text-sm text-zinc-500"
                        >
                          <span className="text-green-500 mt-0.5">&#8226;</span>
                          {d}
                        </li>
                      ))}
                    </ul>
                  </div>
                </div>
              </div>
            </StaggerItem>
          ))}
        </StaggerContainer>
      </div>
    </div>
  );
}
