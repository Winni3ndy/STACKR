"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useState } from "react";

const sections = [
  {
    title: "Getting Started",
    items: [
      { label: "Overview", href: "/docs" },
      { label: "Quickstart", href: "/docs/quickstart" },
    ],
  },
  {
    title: "Reference",
    items: [
      { label: "API Reference", href: "/docs/api" },
      { label: "Developer Guide", href: "/docs/guide" },
    ],
  },
  {
    title: "Learn More",
    items: [
      { label: "Security", href: "/docs/security" },
      { label: "Self-Hosting", href: "/docs/deployment" },
    ],
  },
];

function SidebarContent({ onNavigate }: { onNavigate?: () => void }) {
  const pathname = usePathname();

  return (
    <div className="space-y-6">
      {sections.map((section) => (
        <div key={section.title}>
          <h3 className="text-xs font-semibold text-zinc-500 uppercase tracking-wider mb-2">
            {section.title}
          </h3>
          <ul className="space-y-1">
            {section.items.map((item) => (
              <li key={item.href}>
                <Link
                  href={item.href}
                  onClick={onNavigate}
                  className={`block px-3 py-1.5 text-sm rounded-md transition-colors ${
                    pathname === item.href
                      ? "text-white bg-zinc-800"
                      : "text-zinc-400 hover:text-white hover:bg-zinc-800/50"
                  }`}
                >
                  {item.label}
                </Link>
              </li>
            ))}
          </ul>
        </div>
      ))}
    </div>
  );
}

export function DocsSidebar() {
  return (
    <nav className="w-56 shrink-0 hidden lg:block">
      <div className="sticky top-24">
        <SidebarContent />
      </div>
    </nav>
  );
}

export function DocsMobileNav() {
  const [open, setOpen] = useState(false);
  const pathname = usePathname();

  // Find current page label
  const current = sections
    .flatMap((s) => s.items)
    .find((item) => item.href === pathname);

  return (
    <div className="lg:hidden mb-6">
      <button
        onClick={() => setOpen(!open)}
        className="w-full flex items-center justify-between px-4 py-3 rounded-lg border border-zinc-800 bg-zinc-900/30 text-sm"
      >
        <span className="text-zinc-300">{current?.label || "Docs Navigation"}</span>
        <svg
          className={`w-4 h-4 text-zinc-500 transition-transform ${open ? "rotate-180" : ""}`}
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </svg>
      </button>
      {open && (
        <div className="mt-2 p-4 rounded-lg border border-zinc-800 bg-zinc-900/50">
          <SidebarContent onNavigate={() => setOpen(false)} />
        </div>
      )}
    </div>
  );
}
