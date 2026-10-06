import type { Metadata } from "next";
import { FadeIn, StaggerContainer, StaggerItem } from "@/components/motion-wrapper";

export const metadata: Metadata = {
  title: "Contact",
};

export default function ContactPage() {
  return (
    <div className="pt-32 pb-20">
      <div className="mx-auto max-w-3xl px-6">
        <FadeIn>
          <h1 className="text-4xl sm:text-5xl font-bold tracking-tight mb-6">
            Contact
          </h1>
          <p className="text-lg text-zinc-400 mb-12">
            Questions, contributions, or interested in deploying Stackr?
          </p>
        </FadeIn>

        <StaggerContainer className="space-y-6">
          <StaggerItem>
            <a
              href="https://github.com/AkemiHomura-maworshi/stackr-standard"
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-4 p-5 rounded-xl border border-zinc-800 hover:border-zinc-700 bg-zinc-900/30 hover:bg-zinc-900/60 transition-all group"
            >
              <div className="w-12 h-12 rounded-lg bg-zinc-800 flex items-center justify-center group-hover:bg-zinc-700 transition-colors">
                <svg className="w-6 h-6 text-white" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
                </svg>
              </div>
              <div>
                <h2 className="text-lg font-semibold text-white group-hover:text-green-500 transition-colors">
                  GitHub
                </h2>
                <p className="text-sm text-zinc-400">
                  Source code, issues, and contributions
                </p>
              </div>
              <svg
                className="w-5 h-5 text-zinc-600 ml-auto"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
              </svg>
            </a>
          </StaggerItem>

          <StaggerItem>
            <a
              href="https://github.com/AkemiHomura-maworshi/stackr-standard/issues"
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-4 p-5 rounded-xl border border-zinc-800 hover:border-zinc-700 bg-zinc-900/30 hover:bg-zinc-900/60 transition-all group"
            >
              <div className="w-12 h-12 rounded-lg bg-zinc-800 flex items-center justify-center group-hover:bg-zinc-700 transition-colors">
                <svg className="w-6 h-6 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              </div>
              <div>
                <h2 className="text-lg font-semibold text-white group-hover:text-green-500 transition-colors">
                  Report an Issue
                </h2>
                <p className="text-sm text-zinc-400">
                  Bug reports and feature requests
                </p>
              </div>
              <svg
                className="w-5 h-5 text-zinc-600 ml-auto"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
              </svg>
            </a>
          </StaggerItem>

          <StaggerItem>
            <div className="flex items-center gap-4 p-5 rounded-xl border border-zinc-800 bg-zinc-900/30">
              <div className="w-12 h-12 rounded-lg bg-zinc-800 flex items-center justify-center">
                <svg className="w-6 h-6 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                </svg>
              </div>
              <div>
                <h2 className="text-lg font-semibold text-white">
                  Operator Inquiries
                </h2>
                <p className="text-sm text-zinc-400">
                  Interested in deploying Stackr for your market? Open a GitHub discussion or reach out via issues.
                </p>
              </div>
            </div>
          </StaggerItem>
        </StaggerContainer>

        <FadeIn className="mt-16">
          <div className="p-6 rounded-xl border border-zinc-800 bg-zinc-900/30">
            <h2 className="text-lg font-semibold text-white mb-3">
              Contributing
            </h2>
            <p className="text-sm text-zinc-400 leading-relaxed mb-4">
              Stackr is MIT licensed and welcomes contributions. The codebase
              is written in Rust and follows standard Cargo conventions.
            </p>
            <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4">
              <pre className="font-mono text-sm text-zinc-300">
{`git clone https://github.com/AkemiHomura-maworshi/stackr-standard.git
cd stackr-standard
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test`}
              </pre>
            </div>
          </div>
        </FadeIn>
      </div>
    </div>
  );
}
