"use client";

import { useState, useRef, useEffect, useCallback } from "react";
import { ussdTree, type UssdNode } from "@/lib/ussd-flows";

interface LogEntry {
  direction: "send" | "recv" | "info";
  text: string;
  meta: string;
}

export function UssdSimulator({ compact = false }: { compact?: boolean }) {
  const [started, setStarted] = useState(false);
  const [ended, setEnded] = useState(false);
  const [displayText, setDisplayText] = useState("Press Dial to start a USSD session.");
  const [input, setInput] = useState("");
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [step, setStep] = useState(0);
  const [nodeStack, setNodeStack] = useState<UssdNode[]>([]);
  const inputRef = useRef<HTMLInputElement>(null);
  const bodyRef = useRef<HTMLDivElement>(null);
  const [time, setTime] = useState("");

  useEffect(() => {
    const update = () => {
      const now = new Date();
      setTime(
        now.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })
      );
    };
    update();
    const interval = setInterval(update, 30000);
    return () => clearInterval(interval);
  }, []);

  const addLog = useCallback((entry: LogEntry) => {
    setLogs((prev) => [...prev, entry]);
  }, []);

  const navigate = useCallback(
    (node: UssdNode) => {
      setDisplayText(node.text);
      if (node.end) {
        setEnded(true);
        addLog({
          direction: "recv",
          text: node.text.substring(0, 80),
          meta: "END — session terminated",
        });
      } else {
        addLog({
          direction: "recv",
          text: node.text.substring(0, 80),
          meta: "CON — awaiting input",
        });
        setTimeout(() => inputRef.current?.focus(), 50);
      }
    },
    [addLog]
  );

  const startSession = useCallback(() => {
    setStarted(true);
    setEnded(false);
    setStep(0);
    setNodeStack([ussdTree]);
    setLogs([]);
    addLog({
      direction: "info",
      text: "New session started",
      meta: new Date().toLocaleTimeString(),
    });
    navigate(ussdTree);
  }, [addLog, navigate]);

  const sendInput = useCallback(() => {
    const value = input.trim();
    if (!value) return;

    setInput("");
    setStep((s) => s + 1);

    const currentNode = nodeStack[nodeStack.length - 1];
    addLog({
      direction: "send",
      text: `input="${value}"`,
      meta: `step=${step + 1}`,
    });

    if (currentNode?.options) {
      const next = currentNode.options[value] || currentNode.options["*"];
      if (next) {
        setNodeStack((prev) => [...prev, next]);
        navigate(next);
      } else {
        setDisplayText("Invalid option. Please try again.");
        addLog({
          direction: "recv",
          text: "Invalid option",
          meta: "CON — awaiting input",
        });
        setTimeout(() => inputRef.current?.focus(), 50);
      }
    }
  }, [input, nodeStack, step, addLog, navigate]);

  const phoneHeight = compact ? "h-[580px]" : "h-[680px]";

  return (
    <div className={compact ? "" : "flex gap-10 items-start"}>
      {/* Phone frame */}
      <div
        className={`w-full max-w-[300px] sm:max-w-[340px] ${phoneHeight} bg-zinc-900 rounded-[32px] sm:rounded-[40px] border-2 border-zinc-700 p-3 sm:p-4 flex flex-col shrink-0 shadow-2xl shadow-black/50 mx-auto`}
      >
        {/* Notch */}
        <div className="w-28 h-6 bg-[#09090b] rounded-b-2xl mx-auto mb-2" />

        {/* Status bar */}
        <div className="flex justify-between items-center px-3 pb-2 text-xs text-zinc-500">
          <span className="font-semibold text-white">{time}</span>
          <span className="text-zinc-600">Stackr USSD</span>
          <span>100%</span>
        </div>

        {/* USSD dialog */}
        <div className="flex-1 flex items-center justify-center px-3">
          <div className="w-full bg-zinc-800/80 rounded-xl border border-zinc-700 overflow-hidden flex flex-col max-h-full">
            {/* Header */}
            <div className="bg-zinc-800 px-4 py-3 text-xs text-zinc-500 border-b border-zinc-700 flex justify-between items-center">
              <span className="font-bold text-green-500 text-sm">*384#</span>
              <span>
                {!started
                  ? "Ready"
                  : ended
                    ? "Session Ended"
                    : `Step ${step}`}
              </span>
            </div>

            {/* Body */}
            <div
              ref={bodyRef}
              className={`px-4 py-4 text-sm leading-relaxed whitespace-pre-wrap overflow-y-auto flex-1 min-h-[180px] max-h-[300px] ${
                ended ? "text-amber-400" : "text-zinc-200"
              }`}
            >
              {displayText}
            </div>

            {/* Input area */}
            {!started && (
              <div className="border-t border-zinc-700 p-3">
                <button
                  onClick={startSession}
                  className="w-full bg-green-500 hover:bg-green-600 text-black font-bold py-2.5 rounded-lg transition-colors text-sm"
                >
                  Dial *384#
                </button>
              </div>
            )}

            {started && !ended && (
              <div className="border-t border-zinc-700 p-3 flex gap-2">
                <input
                  ref={inputRef}
                  type="text"
                  value={input}
                  onChange={(e) => setInput(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && sendInput()}
                  placeholder="Enter response..."
                  className="flex-1 bg-zinc-800 border border-zinc-600 rounded-lg px-3 py-2 text-white text-sm outline-none focus:border-green-500 placeholder:text-zinc-600"
                />
                <button
                  onClick={sendInput}
                  className="bg-green-500 hover:bg-green-600 text-black font-bold px-4 py-2 rounded-lg transition-colors text-sm"
                >
                  Send
                </button>
              </div>
            )}

            {started && ended && (
              <div className="border-t border-zinc-700 p-3">
                <button
                  onClick={startSession}
                  className="w-full bg-green-500 hover:bg-green-600 text-black font-bold py-2.5 rounded-lg transition-colors text-sm"
                >
                  New Session
                </button>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Log panel */}
      {!compact && (
        <div className="hidden lg:block w-80">
          <h3 className="text-xs text-zinc-500 uppercase tracking-wider font-semibold mb-3">
            Session Log
          </h3>
          <div className="bg-zinc-950 border border-zinc-800 rounded-lg p-3 max-h-[400px] overflow-y-auto font-mono text-[11px] leading-relaxed space-y-1">
            {logs.length === 0 && (
              <p className="text-zinc-600">
                Start a session to see requests.
              </p>
            )}
            {logs.map((entry, i) => (
              <div key={i} className="border-b border-zinc-900 pb-1">
                <span
                  className={
                    entry.direction === "send"
                      ? "text-green-500"
                      : entry.direction === "recv"
                        ? "text-amber-400"
                        : "text-zinc-500"
                  }
                >
                  {entry.direction === "send"
                    ? "\u2192"
                    : entry.direction === "recv"
                      ? "\u2190"
                      : "\u2022"}
                </span>{" "}
                <span className="text-zinc-300">{entry.text}</span>
                <br />
                <span className="text-zinc-600">{entry.meta}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
