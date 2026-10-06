"use client";

import { useAuth } from "@/lib/auth";
import { generateUsageData, endpointBreakdown, planLimits } from "@/lib/mock-data";
import { useMemo } from "react";

export default function UsagePage() {
  const { user } = useAuth();
  const usageData = useMemo(() => generateUsageData(30), []);

  if (!user) return null;

  const plan = planLimits[user.plan];
  const totalCalls = usageData.reduce((s, d) => s + d.calls, 0);
  const maxDay = Math.max(...usageData.map((d) => d.calls));
  const usagePct = Math.min(100, Math.round((totalCalls / plan.calls) * 100));

  return (
    <div>
      <h1 className="text-2xl font-bold tracking-tight mb-1">Usage</h1>
      <p className="text-zinc-400 text-sm mb-8">
        Monitor your API usage and quota consumption.
      </p>

      {/* Quota bar */}
      <div className="rounded-xl border border-zinc-800 bg-zinc-900/30 p-6 mb-8">
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-sm font-semibold text-zinc-300">Monthly Quota</h2>
          <span className="text-sm text-zinc-400">
            {totalCalls.toLocaleString()} / {plan.calls.toLocaleString()} calls
          </span>
        </div>
        <div className="w-full h-3 bg-zinc-800 rounded-full overflow-hidden">
          <div
            className={`h-full rounded-full transition-all ${usagePct > 80 ? "bg-amber-500" : "bg-green-500"}`}
            style={{ width: `${usagePct}%` }}
          />
        </div>
        <p className="text-xs text-zinc-500 mt-2">
          {usagePct}% used &middot; {plan.name} plan &middot; Resets monthly
        </p>
      </div>

      {/* 30-day chart */}
      <div className="rounded-xl border border-zinc-800 bg-zinc-900/30 p-6 mb-8">
        <h2 className="text-sm font-semibold text-zinc-300 mb-4">API Calls (Last 30 Days)</h2>
        <div className="flex items-end gap-1 h-40">
          {usageData.map(({ date, calls }) => (
            <div key={date} className="flex-1 flex flex-col items-center gap-1 group relative">
              <div className="absolute -top-6 hidden group-hover:block bg-zinc-800 px-2 py-1 rounded text-[10px] text-white whitespace-nowrap z-10">
                {date}: {calls}
              </div>
              <div
                className="w-full bg-green-500/70 hover:bg-green-500 rounded-t transition-colors cursor-default"
                style={{ height: `${maxDay > 0 ? (calls / maxDay) * 100 : 0}%`, minHeight: calls > 0 ? 2 : 0 }}
              />
            </div>
          ))}
        </div>
      </div>

      {/* Endpoint breakdown */}
      <div className="rounded-xl border border-zinc-800 bg-zinc-900/30 p-6">
        <h2 className="text-sm font-semibold text-zinc-300 mb-4">Endpoint Breakdown</h2>
        <div className="space-y-3">
          {endpointBreakdown.map(({ endpoint, calls, pct }) => (
            <div key={endpoint}>
              <div className="flex items-center justify-between mb-1">
                <span className="text-sm font-mono text-zinc-400">{endpoint}</span>
                <span className="text-xs text-zinc-500">{calls} calls ({pct}%)</span>
              </div>
              <div className="w-full h-2 bg-zinc-800 rounded-full overflow-hidden">
                <div
                  className="h-full bg-green-500/70 rounded-full"
                  style={{ width: `${pct}%` }}
                />
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
