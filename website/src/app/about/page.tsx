import type { Metadata } from "next";
import Link from "next/link";
import { FadeIn, StaggerContainer, StaggerItem } from "@/components/motion-wrapper";

export const metadata: Metadata = {
  title: "About",
};

export default function AboutPage() {
  return (
    <div className="pt-32 pb-20">
      <div className="mx-auto max-w-3xl px-6">
        <FadeIn>
          <h1 className="text-4xl sm:text-5xl font-bold tracking-tight mb-6">
            About Stackr
          </h1>
        </FadeIn>

        <div className="space-y-12 text-zinc-400 leading-relaxed">
          <FadeIn delay={0.1}>
            <section>
              <h2 className="text-2xl font-bold text-white mb-4">Mission</h2>
              <p>
                Most of Africa transacts on USSD — not apps, not browsers.
                Stackr is the API that brings stablecoin payments to the
                channel people already use. Developers integrate our API, and
                their users get access to USDC transfers, bill payments, and
                cash-in/cash-out — from any phone. No smartphone required. No
                data plan required. Just a phone number and a 4-digit PIN.
              </p>
            </section>
          </FadeIn>

          <FadeIn delay={0.15}>
            <section>
              <h2 className="text-2xl font-bold text-white mb-4">
                Why Stellar?
              </h2>
              <p className="mb-4">
                USSD sessions timeout after ~30 seconds. That constraint
                eliminates most blockchains. Stellar was chosen because:
              </p>
              <ul className="space-y-2">
                {[
                  "3-5 second finality — fast enough for USSD",
                  "Near-zero transaction fees ($0.00001)",
                  "Built-in DEX for token swaps",
                  "SEP-24 protocol for standardized fiat on/off ramps",
                  "USDC natively issued on Stellar by Circle",
                  "Fee bump transactions — users never need to hold XLM",
                ].map((item) => (
                  <li key={item} className="flex items-start gap-2">
                    <span className="text-green-500 mt-0.5">&#8226;</span>
                    {item}
                  </li>
                ))}
              </ul>
            </section>
          </FadeIn>

          <FadeIn>
            <section>
              <h2 className="text-2xl font-bold text-white mb-4">Why Rust?</h2>
              <p>
                Financial infrastructure has zero tolerance for memory bugs,
                data races, or panics under load. Rust gives us memory safety
                without garbage collection, fearless concurrency with Tokio,
                and the performance to handle USSD&apos;s latency constraints.
                The type system catches entire categories of bugs at compile
                time.
              </p>
            </section>
          </FadeIn>

          <FadeIn>
            <section>
              <h2 className="text-2xl font-bold text-white mb-4">
                Open Source
              </h2>
              <p className="mb-4">
                Stackr is MIT licensed. The entire codebase — wallet
                derivation, transaction signing, state machine, API — is open
                for audit, contribution, and deployment.
              </p>
              <p>
                The goal is infrastructure that any operator can deploy to
                serve their market. Change two environment variables
                (FIAT_CURRENCY and FIAT_COUNTRY) and you have a stablecoin
                payment system for a new country.
              </p>
            </section>
          </FadeIn>

          <FadeIn>
            <section>
              <h2 className="text-2xl font-bold text-white mb-4">
                Tech Stack
              </h2>
              <StaggerContainer className="grid sm:grid-cols-2 gap-4">
                {[
                  { label: "Language", value: "Rust" },
                  { label: "Web Framework", value: "actix-web 4" },
                  { label: "Async Runtime", value: "Tokio" },
                  { label: "Database", value: "PostgreSQL" },
                  { label: "Cache / Sessions", value: "Redis" },
                  { label: "Blockchain", value: "Stellar" },
                  { label: "Stablecoin", value: "USDC (Circle)" },
                  { label: "USSD Gateway", value: "Africa's Talking" },
                  { label: "Price Feed", value: "CoinGecko" },
                  { label: "Bill Payments", value: "Airbills" },
                ].map(({ label, value }) => (
                  <StaggerItem key={label}>
                    <div className="flex justify-between p-3 rounded-lg border border-zinc-800 bg-zinc-900/30">
                      <span className="text-zinc-500 text-sm">{label}</span>
                      <span className="text-white text-sm font-medium">
                        {value}
                      </span>
                    </div>
                  </StaggerItem>
                ))}
              </StaggerContainer>
            </section>
          </FadeIn>

          <FadeIn>
            <div className="pt-8 border-t border-zinc-800">
              <Link
                href="/docs"
                className="inline-flex items-center gap-2 px-6 py-3 bg-green-500 hover:bg-green-600 text-black font-semibold rounded-lg transition-colors"
              >
                Read the Documentation
                <svg
                  className="w-4 h-4"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M9 5l7 7-7 7"
                  />
                </svg>
              </Link>
            </div>
          </FadeIn>
        </div>
      </div>
    </div>
  );
}
