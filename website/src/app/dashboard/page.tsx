"use client";

import Link from "next/link";
import { useAuth } from "@/lib/auth";
import { generateUsageData, recentActivity, planLimits } from "@/lib/mock-data";
import { useMemo } from "react";

export default function DashboardPage() {
  const { user } = useAuth();
  const usageData = useMemo(() => generateUsageData(14), []);

  if (!user) return null;

  const plan = planLimits[user.plan];
  const totalCalls = usageData.reduce((s, d) => s + d.calls, 0);
  const maxDay = Math.max(...usageData.map((d) => d.calls));

  return (
    <div>
      <h1 className="text-2xl font-bold tracking-tight mb-1">
        Welcome back, {user.name.split(" ")[0]}
      </h1>
      <p className="text-zinc-400 text-sm mb-8">
        Here&apos;s what&apos;s happening with your API.
      </p>

      {/* Quick stats */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
        {[
          { label: "API Calls (14d)", value: totalCalls.toLocaleString() },
          { label: "Plan", value: plan.name },
          { label: "Quota", value: `${plan.calls.toLocaleString()} / mo` },
          { label: "Rate Limit", value: plan.rate },
        ].map(({ label, value }) => (
          <div key={label} className="p-4 rounded-xl border border-zinc-800 bg-zinc-900/30">
            <p className="text-xs text-zinc-500 mb-1">{label}</p>
            <p className="text-lg font-bold text-white">{value}</p>
          </div>
        ))}
      </div>

      {/* Usage chart */}
      <div className="rounded-xl border border-zinc-800 bg-zinc-900/30 p-6 mb-8">
        <h2 className="text-sm font-semibold text-zinc-300 mb-4">API Calls (Last 14 Days)</h2>
        <div className="flex items-end gap-1.5 h-32">
          {usageData.map(({ date, calls }) => (
            <div key={date} className="flex-1 flex flex-col items-center gap-1">
              <div
                className="w-full bg-green-500/80 rounded-t"
                style={{ height: `${maxDay > 0 ? (calls / maxDay) * 100 : 0}%`, minHeight: calls > 0 ? 2 : 0 }}
              />
              <span className="text-[9px] text-zinc-600 hidden sm:block">{date.split(" ")[1]}</span>
            </div>
          ))}
        </div>
      </div>

      {/* Recent activity */}
      <div className="rounded-xl border border-zinc-800 bg-zinc-900/30 p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-sm font-semibold text-zinc-300">Recent Activity</h2>
          <Link href="/dashboard/usage" className="text-xs text-green-500 hover:underline">
            View all
          </Link>
        </div>
        <div className="space-y-3">
          {recentActivity.map(({ action, detail, time, status }, i) => (
            <div key={i} className="flex items-center gap-3 text-sm">
              <div className={`w-1.5 h-1.5 rounded-full shrink-0 ${status === "success" ? "bg-green-500" : "bg-red-500"}`} />
              <span className="text-zinc-300 font-medium">{action}</span>
              <span className="text-zinc-500 font-mono text-xs truncate">{detail}</span>
              <span className="text-zinc-600 text-xs ml-auto shrink-0">{time}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
