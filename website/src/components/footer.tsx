import Link from "next/link";

const footerLinks = {
  Product: [
    { label: "Features", href: "/features" },
    { label: "Pricing", href: "/pricing" },
    { label: "How It Works", href: "/how-it-works" },
  ],
  Developers: [
    { label: "Documentation", href: "/docs" },
    { label: "API Reference", href: "/docs/api" },
    { label: "Quickstart", href: "/docs/quickstart" },
  ],
  Company: [
    { label: "About", href: "/about" },
    { label: "Contact", href: "/contact" },
    { label: "GitHub", href: "https://github.com/AkemiHomura-maworshi/stackr-standard" },
  ],
};

export function Footer() {
  return (
    <footer className="border-t border-zinc-800/60 bg-[#0c0c0f]">
      <div className="mx-auto max-w-6xl px-6 py-16">
        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-8">
          <div className="sm:col-span-2 md:col-span-1">
            <Link href="/" className="text-lg font-bold tracking-tight">
              <span className="text-green-500">$</span> stackr
            </Link>
            <p className="mt-3 text-sm text-zinc-500 leading-relaxed">
              Stablecoin payments from any phone in Africa. Built on Stellar.
            </p>
          </div>

          {Object.entries(footerLinks).map(([heading, items]) => (
            <div key={heading}>
              <h3 className="text-sm font-semibold text-zinc-300 mb-3">
                {heading}
              </h3>
              <ul className="space-y-2">
                {items.map(({ label, href }) => {
                  const external = href.startsWith("http");
                  const El = external ? "a" : Link;
                  return (
                    <li key={label}>
                      <El
                        href={href}
                        {...(external
                          ? { target: "_blank", rel: "noopener noreferrer" }
                          : {})}
                        className="text-sm text-zinc-500 hover:text-zinc-300 transition-colors"
                      >
                        {label}
                      </El>
                    </li>
                  );
                })}
              </ul>
            </div>
          ))}
        </div>

        <div className="mt-12 pt-8 border-t border-zinc-800 flex flex-col sm:flex-row justify-between items-center gap-4">
          <p className="text-xs text-zinc-600">
            Stackr &middot; Stablecoin infrastructure for Africa
          </p>
          <p className="text-xs text-zinc-600">Built with Rust &amp; Stellar</p>
        </div>
      </div>
    </footer>
  );
}
