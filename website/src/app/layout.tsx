import type { Metadata } from "next";
import { Inter, JetBrains_Mono } from "next/font/google";
import "./globals.css";
import { Nav } from "@/components/nav";
import { Footer } from "@/components/footer";
import { AuthProvider } from "@/lib/auth";

const inter = Inter({
  subsets: ["latin"],
  variable: "--font-inter",
});

const jetbrains = JetBrains_Mono({
  subsets: ["latin"],
  variable: "--font-jetbrains",
});

export const metadata: Metadata = {
  title: {
    default: "Stackr — USSD Stablecoin Payments on Stellar",
    template: "%s | Stackr",
  },
  description:
    "Open-source USSD stablecoin settlement infrastructure on Stellar. Send money, pay bills, and access DeFi from any phone — no app, no internet required.",
  keywords: [
    "USSD",
    "stablecoin",
    "Stellar",
    "payments",
    "Africa",
    "mobile money",
    "USDC",
  ],
  openGraph: {
    title: "Stackr — USSD Stablecoin Payments on Stellar",
    description:
      "Send money, pay bills, and access DeFi from any phone — no app, no internet required.",
    type: "website",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark">
      <body
        className={`${inter.variable} ${jetbrains.variable} font-sans antialiased bg-[#0c0c0f] text-zinc-100`}
      >
        <AuthProvider>
          <Nav />
          <main className="min-h-screen">{children}</main>
          <Footer />
        </AuthProvider>
      </body>
    </html>
  );
}
