import { createClient, type SupabaseClient } from "@supabase/supabase-js";

const supabaseUrl = process.env.NEXT_PUBLIC_SUPABASE_URL || "";
const supabaseAnonKey = process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY || "";

// Lazy initialization to avoid crashing during static export
// when env vars are not yet set
let _supabase: SupabaseClient | null = null;

export function getSupabase(): SupabaseClient {
  if (_supabase) return _supabase;

  if (!supabaseUrl || !supabaseUrl.startsWith("http")) {
    // Return a dummy client that won't crash during static build
    // At runtime in the browser, the real env vars will be baked in
    _supabase = createClient("https://placeholder.supabase.co", "placeholder");
  } else {
    _supabase = createClient(supabaseUrl, supabaseAnonKey);
  }

  return _supabase;
}

// For convenience — most code imports this
export const supabase = typeof window !== "undefined"
  ? (() => {
      // In browser, always create with real values
      if (supabaseUrl && supabaseUrl.startsWith("http")) {
        return createClient(supabaseUrl, supabaseAnonKey);
      }
      // Fallback during dev without env vars
      return createClient("https://placeholder.supabase.co", "placeholder");
    })()
  : createClient(
      supabaseUrl && supabaseUrl.startsWith("http")
        ? supabaseUrl
        : "https://placeholder.supabase.co",
      supabaseAnonKey || "placeholder"
    );
