// Mock usage stats for the dashboard

export function generateUsageData(days = 30) {
  const data: { date: string; calls: number }[] = [];
  const now = new Date();
  for (let i = days - 1; i >= 0; i--) {
    const d = new Date(now);
    d.setDate(d.getDate() - i);
    const label = d.toLocaleDateString("en-US", { month: "short", day: "numeric" });
    // Simulate realistic usage: ramp up over time with some noise
    const base = Math.floor(20 + (days - i) * 1.5);
    const noise = Math.floor(Math.random() * 15) - 5;
    data.push({ date: label, calls: Math.max(0, base + noise) });
  }
  return data;
}

export const endpointBreakdown = [
  { endpoint: "POST /v1/transfers", calls: 342, pct: 34 },
  { endpoint: "GET /v1/balance/:phone", calls: 287, pct: 29 },
  { endpoint: "POST /v1/accounts", calls: 128, pct: 13 },
  { endpoint: "POST /v1/bills/airtime", calls: 95, pct: 10 },
  { endpoint: "GET /v1/rates", calls: 78, pct: 8 },
  { endpoint: "Other", calls: 64, pct: 6 },
];

export const recentActivity = [
  { action: "API call", detail: "POST /v1/transfers", time: "2 min ago", status: "success" as const },
  { action: "API call", detail: "GET /v1/balance/+250781234567", time: "5 min ago", status: "success" as const },
  { action: "API call", detail: "POST /v1/transfers", time: "12 min ago", status: "error" as const },
  { action: "API key created", detail: "sk_test_...a3f9", time: "1 hour ago", status: "success" as const },
  { action: "API call", detail: "GET /v1/rates", time: "2 hours ago", status: "success" as const },
];

export const planLimits = {
  free: { name: "Free", calls: 1000, rate: "60 req/min" },
  pro: { name: "Pro", calls: 50000, rate: "300 req/min" },
};
