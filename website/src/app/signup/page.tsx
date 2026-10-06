"use client";

import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { useState, Suspense } from "react";
import { useAuth } from "@/lib/auth";

function SignupForm() {
  const searchParams = useSearchParams();
  const { signup } = useAuth();
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [confirmEmail, setConfirmEmail] = useState(false);

  const plan = searchParams.get("plan") || "free";

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");

    if (!name.trim() || !email.trim() || !password.trim()) {
      setError("All fields are required");
      return;
    }
    if (password.length < 6) {
      setError("Password must be at least 6 characters");
      return;
    }

    setSubmitting(true);
    const result = await signup(name.trim(), email.trim(), password);
    setSubmitting(false);

    if (result.ok) {
      // Supabase may require email confirmation
      // If auto-confirm is off, show confirmation message
      // If auto-confirm is on, redirect to dashboard
      setConfirmEmail(true);
    } else {
      setError(result.error || "Signup failed");
    }
  }

  if (confirmEmail) {
    return (
      <div className="pt-32 pb-20 flex items-start justify-center min-h-screen">
        <div className="w-full max-w-md px-6 text-center">
          <Link href="/" className="text-2xl font-bold tracking-tight">
            <span className="text-green-500">$</span> stackr
          </Link>
          <div className="mt-8 p-6 rounded-xl border border-green-500/30 bg-green-500/[0.05]">
            <div className="w-12 h-12 rounded-full bg-green-500/10 flex items-center justify-center mx-auto mb-4">
              <svg className="w-6 h-6 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
              </svg>
            </div>
            <h2 className="text-xl font-bold text-white mb-2">Check your email</h2>
            <p className="text-sm text-zinc-400 mb-4">
              We sent a confirmation link to <strong className="text-zinc-300">{email}</strong>.
              Click the link to activate your account.
            </p>
            <p className="text-xs text-zinc-500">
              Didn&apos;t get it? Check your spam folder, or{" "}
              <button
                onClick={() => setConfirmEmail(false)}
                className="text-green-500 hover:underline"
              >
                try again
              </button>.
            </p>
          </div>
          <p className="mt-6 text-sm text-zinc-500">
            Already confirmed?{" "}
            <Link href="/login" className="text-green-500 hover:underline font-medium">
              Log in
            </Link>
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="pt-32 pb-20 flex items-start justify-center min-h-screen">
      <div className="w-full max-w-md px-6">
        <div className="text-center mb-8">
          <Link href="/" className="text-2xl font-bold tracking-tight">
            <span className="text-green-500">$</span> stackr
          </Link>
          <h1 className="mt-6 text-3xl font-bold tracking-tight">
            Create your account
          </h1>
          <p className="mt-2 text-zinc-400">
            {plan === "pro"
              ? "Start your Pro trial — no credit card required."
              : "Free tier includes 1,000 API calls per month."}
          </p>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          {error && (
            <div className="p-3 rounded-lg bg-red-500/10 border border-red-500/30 text-red-400 text-sm">
              {error}
            </div>
          )}

          <div>
            <label htmlFor="name" className="block text-sm font-medium text-zinc-300 mb-1.5">
              Name
            </label>
            <input
              id="name"
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full rounded-lg border border-zinc-700 bg-zinc-900 px-4 py-2.5 text-white text-sm outline-none focus:border-green-500 placeholder:text-zinc-600"
              placeholder="Jane Doe"
            />
          </div>

          <div>
            <label htmlFor="email" className="block text-sm font-medium text-zinc-300 mb-1.5">
              Email
            </label>
            <input
              id="email"
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              className="w-full rounded-lg border border-zinc-700 bg-zinc-900 px-4 py-2.5 text-white text-sm outline-none focus:border-green-500 placeholder:text-zinc-600"
              placeholder="you@example.com"
            />
          </div>

          <div>
            <label htmlFor="password" className="block text-sm font-medium text-zinc-300 mb-1.5">
              Password
            </label>
            <input
              id="password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              className="w-full rounded-lg border border-zinc-700 bg-zinc-900 px-4 py-2.5 text-white text-sm outline-none focus:border-green-500 placeholder:text-zinc-600"
              placeholder="At least 6 characters"
            />
          </div>

          <button
            type="submit"
            disabled={submitting}
            className="w-full py-2.5 bg-green-500 hover:bg-green-600 disabled:opacity-50 disabled:cursor-not-allowed text-black font-semibold rounded-lg transition-colors text-sm"
          >
            {submitting ? "Creating account..." : "Create Account"}
          </button>
        </form>

        <p className="mt-6 text-center text-sm text-zinc-500">
          Already have an account?{" "}
          <Link href="/login" className="text-green-500 hover:underline font-medium">
            Log in
          </Link>
        </p>
      </div>
    </div>
  );
}

export default function SignupPage() {
  return (
    <Suspense>
      <SignupForm />
    </Suspense>
  );
}
