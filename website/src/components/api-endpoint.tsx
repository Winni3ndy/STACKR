"use client";

import { useState } from "react";
import type { ApiEndpoint } from "@/lib/api-data";

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);

  const copy = () => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <button
      onClick={copy}
      className="absolute top-2 right-2 px-2 py-1 text-[10px] text-zinc-500 hover:text-zinc-300 bg-zinc-900 border border-zinc-700 rounded transition-colors"
      title="Copy to clipboard"
    >
      {copied ? "Copied" : "Copy"}
    </button>
  );
}

export function ApiEndpointCard({ endpoint }: { endpoint: ApiEndpoint }) {
  const [open, setOpen] = useState(false);

  return (
    <div className="border border-zinc-800/60 rounded-lg overflow-hidden">
      <button
        onClick={() => setOpen(!open)}
        className="w-full flex items-center gap-3 px-4 py-3.5 text-left hover:bg-zinc-900/40 transition-colors"
      >
        <span
          className={`text-xs font-bold px-2 py-0.5 rounded shrink-0 ${
            endpoint.method === "GET"
              ? "bg-blue-500/10 text-blue-400 border border-blue-500/20"
              : "bg-green-500/10 text-green-400 border border-green-500/20"
          }`}
        >
          {endpoint.method}
        </span>
        <code className="text-sm text-zinc-300 font-mono truncate">{endpoint.path}</code>
        <span className="hidden sm:inline text-xs text-zinc-600 ml-2">{endpoint.title}</span>
        {endpoint.auth && (
          <span className="ml-auto shrink-0 text-[10px] text-amber-500/80 border border-amber-500/20 bg-amber-500/5 px-1.5 py-0.5 rounded font-medium">
            AUTH
          </span>
        )}
        <svg
          className={`w-4 h-4 text-zinc-500 transition-transform shrink-0 ml-2 ${open ? "rotate-180" : ""}`}
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </svg>
      </button>

      {open && (
        <div className="border-t border-zinc-800/60 px-4 py-5 space-y-5 bg-zinc-950/30">
          <p className="text-sm text-zinc-400 leading-relaxed">{endpoint.description}</p>

          {/* curl */}
          <div>
            <h4 className="text-xs font-semibold text-zinc-500 uppercase tracking-wider mb-2">
              curl
            </h4>
            <div className="relative">
              <pre className="bg-zinc-950 border border-zinc-800 rounded-lg p-3 pr-16 overflow-x-auto text-xs font-mono text-zinc-400 leading-relaxed">
                {endpoint.curl}
              </pre>
              <CopyButton text={endpoint.curl} />
            </div>
          </div>

          {/* Request body */}
          {endpoint.request && (
            <div>
              <h4 className="text-xs font-semibold text-zinc-500 uppercase tracking-wider mb-2">
                Request Body
                <span className="ml-2 font-normal text-zinc-600">{endpoint.request.contentType}</span>
              </h4>
              <div className="relative">
                <pre className="bg-zinc-950 border border-zinc-800 rounded-lg p-3 pr-16 overflow-x-auto text-xs font-mono text-zinc-400">
                  {endpoint.request.body}
                </pre>
                <CopyButton text={endpoint.request.body} />
              </div>
            </div>
          )}

          {/* Success response */}
          <div>
            <h4 className="text-xs font-semibold text-zinc-500 uppercase tracking-wider mb-2">
              Response
              <span className="ml-2 text-green-500/80 font-mono">{endpoint.response.status}</span>
            </h4>
            {endpoint.response.body ? (
              <div className="relative">
                <pre className="bg-zinc-950 border border-zinc-800 rounded-lg p-3 pr-16 overflow-x-auto text-xs font-mono text-zinc-400">
                  {endpoint.response.body}
                </pre>
                <CopyButton text={endpoint.response.body} />
              </div>
            ) : (
              <p className="text-xs text-zinc-600 italic">Empty response body</p>
            )}
          </div>

          {/* Errors */}
          {endpoint.errors.length > 0 && (
            <div>
              <h4 className="text-xs font-semibold text-zinc-500 uppercase tracking-wider mb-2">
                Error Responses
              </h4>
              <div className="border border-zinc-800 rounded-lg overflow-hidden">
                {endpoint.errors.map(({ status, description }) => (
                  <div
                    key={status}
                    className="flex items-start gap-3 px-3 py-2 text-xs border-b border-zinc-800/50 last:border-0"
                  >
                    <span className="font-mono text-red-400/80 shrink-0 mt-px">{status}</span>
                    <span className="text-zinc-400">{description}</span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
