"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useState } from "react";
import { useAuth } from "@/lib/auth";

const links = [
  { href: "/features", label: "Features" },
  { href: "/pricing", label: "Pricing" },
  { href: "/how-it-works", label: "How It Works" },
  { href: "/docs", label: "Docs" },
];

export function Nav() {
  const pathname = usePathname();
  const [open, setOpen] = useState(false);
  const { user } = useAuth();

  // Hide nav on dashboard pages (dashboard has its own sidebar)
  if (pathname.startsWith("/dashboard")) return null;

  return (
    <nav className="fixed top-0 z-50 w-full border-b border-zinc-800/60 bg-[#0c0c0f]/80 backdrop-blur-md">
      <div className="mx-auto flex h-16 max-w-6xl items-center justify-between px-6">
        <Link href="/" className="flex items-center gap-2 text-lg font-bold tracking-tight">
          <span className="text-green-500">$</span> stackr
        </Link>

        {/* Desktop */}
        <div className="hidden md:flex items-center gap-1">
          {links.map(({ href, label }) => {
            const active =
              href === "/" ? pathname === "/" : pathname.startsWith(href);
            return (
              <Link
                key={href}
                href={href}
                className={`px-3 py-2 text-sm rounded-md transition-colors ${
                  active
                    ? "text-white bg-zinc-800"
                    : "text-zinc-400 hover:text-white hover:bg-zinc-800/50"
                }`}
              >
                {label}
              </Link>
            );
          })}
        </div>

        <div className="hidden md:flex items-center gap-3">
          {user ? (
            <>
              <Link
                href="/dashboard"
                className="inline-flex items-center gap-2 px-4 py-2 bg-green-500 hover:bg-green-600 text-black text-sm font-semibold rounded-lg transition-colors"
              >
                <div className="w-5 h-5 rounded-full bg-black/20 flex items-center justify-center text-[10px] font-bold">
                  {user.name.charAt(0).toUpperCase()}
                </div>
                Dashboard
              </Link>
            </>
          ) : (
            <>
              <Link
                href="/login"
                className="text-sm text-zinc-400 hover:text-white transition-colors"
              >
                Log In
              </Link>
              <Link
                href="/signup"
                className="inline-flex items-center gap-2 px-4 py-2 bg-green-500 hover:bg-green-600 text-black text-sm font-semibold rounded-lg transition-colors"
              >
                Get Started Free
              </Link>
            </>
          )}
        </div>

        {/* Mobile toggle */}
        <button
          className="md:hidden p-2 text-zinc-400 hover:text-white"
          onClick={() => setOpen(!open)}
          aria-label="Toggle menu"
        >
          {open ? (
            <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          ) : (
            <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
            </svg>
          )}
        </button>
      </div>

      {/* Mobile menu */}
      {open && (
        <div className="md:hidden border-t border-zinc-800/60 bg-[#0c0c0f] px-6 py-4 space-y-1">
          {links.map(({ href, label }) => {
            const active =
              href === "/" ? pathname === "/" : pathname.startsWith(href);
            return (
              <Link
                key={href}
                href={href}
                onClick={() => setOpen(false)}
                className={`block px-3 py-2 text-sm rounded-md ${
                  active ? "text-white bg-zinc-800" : "text-zinc-400 hover:text-white"
                }`}
              >
                {label}
              </Link>
            );
          })}
          <div className="pt-3 mt-3 border-t border-zinc-800/60 space-y-2">
            {user ? (
              <Link
                href="/dashboard"
                onClick={() => setOpen(false)}
                className="block px-3 py-2 text-sm font-semibold text-green-500"
              >
                Dashboard
              </Link>
            ) : (
              <>
                <Link
                  href="/login"
                  onClick={() => setOpen(false)}
                  className="block px-3 py-2 text-sm text-zinc-400 hover:text-white"
                >
                  Log In
                </Link>
                <Link
                  href="/signup"
                  onClick={() => setOpen(false)}
                  className="block px-3 py-2 text-sm font-semibold text-green-500"
                >
                  Get Started Free
                </Link>
              </>
            )}
          </div>
        </div>
      )}
    </nav>
  );
}
