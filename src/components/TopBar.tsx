import {
  FolderOpen,
  Loader2,
  RefreshCw,
  Sparkles,
  Undo2,
} from "lucide-react";
import type { BatchInfo, BusyState } from "../types";

interface TopBarProps {
  folder: string;
  selectedCount: number;
  batchInfo: BatchInfo | null;
  autoOrganize: boolean;
  busy: BusyState;
  onPickFolder: () => void;
  onRescan: () => void;
  onToggleAutoOrganize: () => void;
  onUndo: () => void;
  onOrganize: () => void;
}

function Spinner() {
  return <Loader2 className="h-4 w-4 animate-spin" aria-hidden />;
}

export function TopBar({
  folder,
  selectedCount,
  batchInfo,
  autoOrganize,
  busy,
  onPickFolder,
  onRescan,
  onToggleAutoOrganize,
  onUndo,
  onOrganize,
}: TopBarProps) {
  const anyBusy = busy !== null;

  return (
    <header className="sticky top-0 z-20 border-b border-white/[0.06] bg-[#0f1117]/90 backdrop-blur-md">
      <div className="mx-auto flex max-w-[1220px] flex-wrap items-center gap-3 px-6 py-4">
        <div className="flex items-center gap-2.5 pr-1">
          <span className="flex h-9 w-9 items-center justify-center rounded-lg bg-sky-500/15 text-sky-400 ring-1 ring-inset ring-sky-400/25">
            <Sparkles className="h-5 w-5" aria-hidden />
          </span>
          <div className="leading-tight">
            <div className="text-[15px] font-semibold tracking-tight text-white">
              DropTidy
            </div>
            <div className="text-[11px] text-slate-500">Tidy downloads, safely</div>
          </div>
        </div>

        <div className="flex min-w-[240px] flex-1 items-center gap-2 rounded-lg border border-white/[0.07] bg-white/[0.03] px-3 py-2">
          <FolderOpen className="h-4 w-4 shrink-0 text-slate-500" aria-hidden />
          <span
            className="min-w-0 flex-1 truncate font-mono text-xs text-slate-300"
            title={folder}
          >
            {folder || "Detecting Downloads folder…"}
          </span>
          <button
            type="button"
            onClick={onPickFolder}
            disabled={anyBusy}
            className="shrink-0 text-xs font-medium text-sky-400 transition-colors hover:text-sky-300 disabled:cursor-not-allowed disabled:opacity-50"
          >
            Change
          </button>
        </div>

        <button
          type="button"
          onClick={onRescan}
          disabled={anyBusy}
          className="icon-btn"
          title="Re-scan the target folder"
        >
          {busy === "scan" ? <Spinner /> : <RefreshCw className="h-4 w-4" aria-hidden />}
          <span>Rescan</span>
        </button>

        <button
          type="button"
          role="switch"
          aria-checked={autoOrganize}
          onClick={onToggleAutoOrganize}
          disabled={anyBusy}
          title={
            autoOrganize
              ? "Auto-Organize is active — new downloads are sorted automatically"
              : "Turn on Auto-Organize"
          }
          className={`inline-flex items-center gap-2 rounded-lg border px-3 py-2 text-sm font-medium transition-all duration-150 disabled:cursor-not-allowed disabled:opacity-40 ${
            autoOrganize
              ? "border-emerald-500/30 bg-emerald-500/10 text-emerald-200"
              : "border-white/[0.08] bg-white/[0.03] text-slate-300 hover:border-white/[0.16] hover:bg-white/[0.07]"
          }`}
        >
          <span className="relative flex h-2.5 w-2.5 items-center justify-center">
            {autoOrganize && (
              <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-400 opacity-75" />
            )}
            <span
              className={`relative inline-flex h-2.5 w-2.5 rounded-full ${
                autoOrganize ? "bg-emerald-400" : "bg-slate-500"
              }`}
            />
          </span>
          <span>Auto-Organize</span>
          <span
            className={`relative inline-flex h-4 w-7 items-center rounded-full transition-colors duration-150 ${
              autoOrganize ? "bg-emerald-500/70" : "bg-white/15"
            }`}
          >
            <span
              className={`h-3 w-3 rounded-full bg-white shadow transition-transform duration-150 ${
                autoOrganize ? "translate-x-3.5" : "translate-x-0.5"
              }`}
            />
          </span>
        </button>

        <button
          type="button"
          onClick={onUndo}
          disabled={anyBusy || !batchInfo}
          className="icon-btn"
          title={batchInfo ? "Undo the most recent sort" : "No sort to undo yet"}
        >
          {busy === "undo" ? (
            <Spinner />
          ) : (
            <Undo2 className="h-4 w-4" aria-hidden />
          )}
          <span>Undo Last Sort</span>
          {batchInfo && (
            <span className="ml-0.5 rounded bg-white/10 px-1.5 py-0.5 text-[10px] font-semibold tabular-nums text-slate-300">
              {batchInfo.fileCount}
            </span>
          )}
        </button>

        <button
          type="button"
          onClick={onOrganize}
          disabled={anyBusy || selectedCount === 0}
          className="primary-btn"
          title="Move selected files into category folders"
        >
          {busy === "organize" ? (
            <Spinner />
          ) : (
            <Sparkles className="h-4 w-4" aria-hidden />
          )}
          <span>Organize Selected</span>
          <span className="rounded bg-white/20 px-1.5 py-0.5 text-[10px] font-semibold tabular-nums">
            {selectedCount}
          </span>
        </button>
      </div>
    </header>
  );
}
