"use client";

import { useState, useEffect, useCallback } from "react";
import { useAuth } from "@/lib/auth";
import { supabase } from "@/lib/supabase";

interface ApiKey {
  id: string;
  name: string;
  key_prefix: string;
  created_at: string;
}

function generateKey() {
  const chars = "abcdefghijklmnopqrstuvwxyz0123456789";
  let result = "sk_test_";
  for (let i = 0; i < 32; i++) {
    result += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return result;
}

export default function ApiKeysPage() {
  const { user } = useAuth();
  const [keys, setKeys] = useState<ApiKey[]>([]);
  const [newKeyName, setNewKeyName] = useState("");
  const [revealedKey, setRevealedKey] = useState<string | null>(null);
  const [copied, setCopied] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [creating, setCreating] = useState(false);

  const fetchKeys = useCallback(async () => {
    if (!user) return;
    const { data } = await supabase
      .from("api_keys")
      .select("id, name, key_prefix, created_at")
      .eq("user_id", user.id)
      .order("created_at", { ascending: false });
    if (data) setKeys(data);
    setLoading(false);
  }, [user]);

  useEffect(() => {
    fetchKeys();
  }, [fetchKeys]);

  async function createKey() {
    if (!user || creating) return;
    setCreating(true);

    const name = newKeyName.trim() || "Default";
    const fullKey = generateKey();
    const keyPrefix = fullKey.substring(0, 12);

    const { error } = await supabase.from("api_keys").insert({
      user_id: user.id,
      name,
      key_hash: fullKey, // In production, hash this with SHA-256
      key_prefix: keyPrefix,
    });

    if (!error) {
      setNewKeyName("");
      setRevealedKey(fullKey);
      await fetchKeys();
    }
    setCreating(false);
  }

  async function deleteKey(id: string) {
    await supabase.from("api_keys").delete().eq("id", id);
    setKeys(keys.filter((k) => k.id !== id));
    if (revealedKey) setRevealedKey(null);
  }

  function copyKey(key: string) {
    navigator.clipboard.writeText(key);
    setCopied(key);
    setTimeout(() => setCopied(null), 2000);
  }

  if (!user) return null;

  return (
    <div>
      <h1 className="text-2xl font-bold tracking-tight mb-1">API Keys</h1>
      <p className="text-zinc-400 text-sm mb-8">
        Create and manage your API keys. Keep them secret — treat them like passwords.
      </p>

      {/* Revealed key warning */}
      {revealedKey && (
        <div className="mb-6 p-4 rounded-xl border border-amber-500/30 bg-amber-500/[0.05]">
          <p className="text-sm text-amber-400 font-medium mb-2">
            Copy your key now — it won&apos;t be shown again.
          </p>
          <div className="flex items-center gap-2">
            <code className="flex-1 bg-zinc-900 rounded-lg px-3 py-2 text-sm font-mono text-zinc-300 overflow-x-auto">
              {revealedKey}
            </code>
            <button
              onClick={() => copyKey(revealedKey)}
              className="shrink-0 px-3 py-2 bg-zinc-800 hover:bg-zinc-700 rounded-lg text-sm text-white transition-colors"
            >
              {copied === revealedKey ? "Copied!" : "Copy"}
            </button>
          </div>
          <button
            onClick={() => setRevealedKey(null)}
            className="mt-2 text-xs text-zinc-500 hover:text-zinc-300"
          >
            Dismiss
          </button>
        </div>
      )}

      {/* Create key */}
      <div className="flex gap-3 mb-8">
        <input
          type="text"
          value={newKeyName}
          onChange={(e) => setNewKeyName(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && createKey()}
          placeholder="Key name (e.g. Production)"
          className="flex-1 max-w-xs rounded-lg border border-zinc-700 bg-zinc-900 px-4 py-2.5 text-white text-sm outline-none focus:border-green-500 placeholder:text-zinc-600"
        />
        <button
          onClick={createKey}
          disabled={creating}
          className="px-4 py-2.5 bg-green-500 hover:bg-green-600 disabled:opacity-50 text-black font-semibold rounded-lg transition-colors text-sm"
        >
          {creating ? "Creating..." : "Create Key"}
        </button>
      </div>

      {/* Keys list */}
      {loading ? (
        <div className="rounded-xl border border-zinc-800 bg-zinc-900/30 p-8 text-center">
          <p className="text-zinc-500 text-sm">Loading keys...</p>
        </div>
      ) : keys.length === 0 ? (
        <div className="rounded-xl border border-zinc-800 bg-zinc-900/30 p-8 text-center">
          <p className="text-zinc-500 text-sm">No API keys yet. Create one above to get started.</p>
        </div>
      ) : (
        <div className="space-y-3">
          {keys.map((k) => (
            <div
              key={k.id}
              className="flex items-center gap-4 p-4 rounded-xl border border-zinc-800 bg-zinc-900/30"
            >
              <div className="flex-1 min-w-0">
                <p className="text-sm font-medium text-white">{k.name}</p>
                <code className="text-xs font-mono text-zinc-500 mt-1 block">
                  {k.key_prefix}...
                </code>
              </div>
              <span className="text-xs text-zinc-600 shrink-0">
                {new Date(k.created_at).toLocaleDateString()}
              </span>
              <button
                onClick={() => deleteKey(k.id)}
                className="text-xs text-red-400 hover:text-red-300 shrink-0"
              >
                Delete
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
