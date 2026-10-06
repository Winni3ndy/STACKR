import type { Metadata } from "next";
import { UssdSimulator } from "@/components/ussd-simulator";
import { FadeIn, StaggerContainer, StaggerItem } from "@/components/motion-wrapper";

export const metadata: Metadata = {
  title: "How It Works",
};

const steps = [
  {
    step: "01",
    title: "User dials *384#",
    description:
      "On any phone — feature phone, smartphone, even a borrowed phone. No app install, no data connection. The telco routes the USSD request to Africa's Talking.",
  },
  {
    step: "02",
    title: "Africa's Talking forwards to Stackr",
    description:
      "Africa's Talking sends a POST request to /ussd/callback with the session ID, phone number, and cumulative user input. IP allowlist and rate limiting are enforced.",
  },
  {
    step: "03",
    title: "USSD state machine processes the request",
    description:
      "Redis stores the session state. Postgres stores user data. The state machine determines which menu to show based on the cumulative text input.",
  },
  {
    step: "04",
    title: "Stellar transaction executes",
    description:
      "For financial operations, Stackr builds an XDR transaction, signs it with the user's derived keypair, wraps it in a fee bump, and submits to Stellar Horizon.",
  },
  {
    step: "05",
    title: "Response sent back to user",
    description:
      "The USSD response (CON for continue, END for terminate) is sent back through Africa's Talking to the telco to the user's phone. Total time: under 3 seconds.",
  },
];

export default function HowItWorksPage() {
  return (
    <div className="pt-32 pb-20">
      <div className="mx-auto max-w-6xl px-6">
        <FadeIn className="text-center mb-16">
          <h1 className="text-4xl sm:text-5xl font-bold tracking-tight">
            How It Works
          </h1>
          <p className="mt-4 text-lg text-zinc-400 max-w-2xl mx-auto">
            From USSD dial to Stellar settlement. Try the interactive
            simulator, then read the architecture walkthrough.
          </p>
        </FadeIn>

        {/* Simulator */}
        <FadeIn delay={0.2} className="flex justify-center mb-24">
          <UssdSimulator />
        </FadeIn>

        {/* Steps */}
        <div className="max-w-3xl mx-auto">
          <FadeIn>
            <h2 className="text-2xl font-bold mb-10">
              Architecture Walkthrough
            </h2>
          </FadeIn>

          <StaggerContainer className="space-y-0">
            {steps.map(({ step, title, description }, i) => (
              <StaggerItem key={step}>
                <div className="flex gap-6">
                  {/* Timeline */}
                  <div className="flex flex-col items-center">
                    <div className="w-10 h-10 rounded-full bg-green-500/10 border border-green-500/30 flex items-center justify-center text-green-500 text-xs font-bold shrink-0">
                      {step}
                    </div>
                    {i < steps.length - 1 && (
                      <div className="w-px h-full bg-zinc-800 my-2" />
                    )}
                  </div>

                  {/* Content */}
                  <div className="pb-12">
                    <h3 className="text-lg font-semibold text-white">
                      {title}
                    </h3>
                    <p className="text-sm text-zinc-400 mt-2 leading-relaxed">
                      {description}
                    </p>
                  </div>
                </div>
              </StaggerItem>
            ))}
          </StaggerContainer>
        </div>

        {/* Architecture diagram */}
        <FadeIn className="mt-16 max-w-3xl mx-auto">
          <h2 className="text-2xl font-bold mb-6">System Architecture</h2>
          <div className="rounded-xl border border-zinc-800 bg-zinc-950 p-6 overflow-x-auto">
            <pre className="font-mono text-sm text-zinc-400 leading-relaxed">
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
          </div>
        </FadeIn>
      </div>
    </div>
  );
}
