"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useState } from "react";
import { useAuth } from "@/lib/auth";

export default function LoginPage() {
  const router = useRouter();
  const { login, resetPassword } = useAuth();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [resetMode, setResetMode] = useState(false);
  const [resetSent, setResetSent] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");

    if (!email.trim()) {
      setError("Email is required");
      return;
    }

    if (resetMode) {
      setSubmitting(true);
      const result = await resetPassword(email.trim());
      setSubmitting(false);
      if (result.ok) {
        setResetSent(true);
      } else {
        setError(result.error || "Failed to send reset email");
      }
      return;
    }

    if (!password.trim()) {
      setError("Password is required");
      return;
    }

    setSubmitting(true);
    const result = await login(email.trim(), password);
    setSubmitting(false);

    if (result.ok) {
      router.push("/dashboard");
    } else {
      setError(result.error || "Login failed");
    }
  }

  return (
    <div className="pt-32 pb-20 flex items-start justify-center min-h-screen">
      <div className="w-full max-w-md px-6">
        <div className="text-center mb-8">
          <Link href="/" className="text-2xl font-bold tracking-tight">
            <span className="text-green-500">$</span> stackr
          </Link>
          <h1 className="mt-6 text-3xl font-bold tracking-tight">
            {resetMode ? "Reset password" : "Welcome back"}
          </h1>
          <p className="mt-2 text-zinc-400">
            {resetMode
              ? "Enter your email and we'll send a reset link."
              : "Sign in to your Stackr dashboard."}
          </p>
        </div>

        {resetSent ? (
          <div className="p-4 rounded-xl border border-green-500/30 bg-green-500/[0.05] text-center">
            <p className="text-sm text-green-400 font-medium mb-2">Reset email sent</p>
            <p className="text-sm text-zinc-400">
              Check your inbox for a password reset link. It may take a minute to arrive.
            </p>
            <button
              onClick={() => { setResetMode(false); setResetSent(false); }}
              className="mt-4 text-sm text-green-500 hover:underline"
            >
              Back to login
            </button>
          </div>
        ) : (
          <>
            <form onSubmit={handleSubmit} className="space-y-4">
              {error && (
                <div className="p-3 rounded-lg bg-red-500/10 border border-red-500/30 text-red-400 text-sm">
                  {error}
                </div>
              )}

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

              {!resetMode && (
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
                    placeholder="Your password"
                  />
                </div>
              )}

              <button
                type="submit"
                disabled={submitting}
                className="w-full py-2.5 bg-green-500 hover:bg-green-600 disabled:opacity-50 disabled:cursor-not-allowed text-black font-semibold rounded-lg transition-colors text-sm"
              >
                {submitting
                  ? (resetMode ? "Sending..." : "Signing in...")
                  : (resetMode ? "Send Reset Link" : "Sign In")}
              </button>
            </form>

            <div className="mt-4 text-center">
              <button
                onClick={() => { setResetMode(!resetMode); setError(""); }}
                className="text-sm text-zinc-500 hover:text-zinc-300 transition-colors"
              >
                {resetMode ? "Back to login" : "Forgot password?"}
              </button>
            </div>

            <p className="mt-4 text-center text-sm text-zinc-500">
              Don&apos;t have an account?{" "}
              <Link href="/signup" className="text-green-500 hover:underline font-medium">
                Sign up free
              </Link>
            </p>
          </>
        )}
      </div>
    </div>
  );
}
