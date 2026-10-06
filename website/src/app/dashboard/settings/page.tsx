"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/lib/auth";
import { planLimits } from "@/lib/mock-data";

export default function SettingsPage() {
  const router = useRouter();
  const { user, updateProfile, deleteAccount } = useAuth();
  const [name, setName] = useState(user?.name || "");
  const [email, setEmail] = useState(user?.email || "");
  const [saved, setSaved] = useState(false);
  const [saving, setSaving] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [deleting, setDeleting] = useState(false);

  if (!user) return null;

  const plan = planLimits[user.plan];

  async function handleSave(e: React.FormEvent) {
    e.preventDefault();
    setSaving(true);
    await updateProfile({ name: name.trim(), email: email.trim() });
    setSaving(false);
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  }

  async function handleDelete() {
    if (!confirmDelete) {
      setConfirmDelete(true);
      return;
    }
    setDeleting(true);
    await deleteAccount();
    router.push("/");
  }

  return (
    <div>
      <h1 className="text-2xl font-bold tracking-tight mb-1">Settings</h1>
      <p className="text-zinc-400 text-sm mb-8">
        Manage your account settings and preferences.
      </p>

      {/* Profile */}
      <div className="rounded-xl border border-zinc-800 bg-zinc-900/30 p-6 mb-8">
        <h2 className="text-sm font-semibold text-zinc-300 mb-4">Profile</h2>
        <form onSubmit={handleSave} className="space-y-4 max-w-md">
          <div>
            <label htmlFor="settings-name" className="block text-sm text-zinc-400 mb-1.5">Name</label>
            <input
              id="settings-name"
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full rounded-lg border border-zinc-700 bg-zinc-900 px-4 py-2.5 text-white text-sm outline-none focus:border-green-500"
            />
          </div>
          <div>
            <label htmlFor="settings-email" className="block text-sm text-zinc-400 mb-1.5">Email</label>
            <input
              id="settings-email"
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              className="w-full rounded-lg border border-zinc-700 bg-zinc-900 px-4 py-2.5 text-white text-sm outline-none focus:border-green-500"
            />
            <p className="text-xs text-zinc-600 mt-1">Changing email requires re-verification.</p>
          </div>
          <button
            type="submit"
            disabled={saving}
            className="px-4 py-2.5 bg-green-500 hover:bg-green-600 disabled:opacity-50 text-black font-semibold rounded-lg transition-colors text-sm"
          >
            {saving ? "Saving..." : saved ? "Saved!" : "Save Changes"}
          </button>
        </form>
      </div>

      {/* Current plan */}
      <div className="rounded-xl border border-zinc-800 bg-zinc-900/30 p-6 mb-8">
        <h2 className="text-sm font-semibold text-zinc-300 mb-4">Current Plan</h2>
        <div className="flex items-center gap-4">
          <div>
            <p className="text-lg font-bold text-white">{plan.name}</p>
            <p className="text-sm text-zinc-400">
              {plan.calls.toLocaleString()} API calls/month &middot; {plan.rate}
            </p>
          </div>
          <span className="ml-auto px-3 py-1 bg-green-500/10 border border-green-500/30 text-green-500 text-xs font-semibold rounded-full">
            Active
          </span>
        </div>
      </div>

      {/* Delete account */}
      <div className="rounded-xl border border-red-500/20 bg-red-500/[0.02] p-6">
        <h2 className="text-sm font-semibold text-red-400 mb-2">Danger Zone</h2>
        <p className="text-sm text-zinc-400 mb-4">
          Permanently delete your account and all associated data. This action cannot be undone.
        </p>
        <button
          onClick={handleDelete}
          disabled={deleting}
          className={`px-4 py-2.5 rounded-lg text-sm font-semibold transition-colors ${
            confirmDelete
              ? "bg-red-500 hover:bg-red-600 text-white"
              : "border border-red-500/30 text-red-400 hover:bg-red-500/10"
          }`}
        >
          {deleting ? "Deleting..." : confirmDelete ? "Click again to confirm deletion" : "Delete Account"}
        </button>
        {confirmDelete && (
          <button
            onClick={() => setConfirmDelete(false)}
            className="ml-3 text-sm text-zinc-500 hover:text-zinc-300"
          >
            Cancel
          </button>
        )}
      </div>
    </div>
  );
}
