import { Hero } from "@/components/hero";
import { StatsBar } from "@/components/stats-bar";
import { HowItWorks } from "@/components/how-it-works";
import { FeatureGrid } from "@/components/feature-grid";
import { CodePreview } from "@/components/code-preview";
import { TrustSignals } from "@/components/trust-signals";
import { Architecture } from "@/components/architecture";
import { CTA } from "@/components/cta";

export default function Home() {
  return (
    <>
      <Hero />
      <StatsBar />
      <HowItWorks />
      <FeatureGrid />
      <CodePreview />
      <TrustSignals />
      <Architecture />
      <CTA />
    </>
  );
}
