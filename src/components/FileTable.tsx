import { FileQuestion, FolderInput, Layers3 } from "lucide-react";
import type { FileEntry } from "../types";
import { CategoryBadge } from "./CategoryBadge";
import { formatDate, formatSize } from "../lib/format";

/** Keep the preview compact: `Installers\setup.exe` instead of a long path. */
function shortDestination(destination: string): string {
  const parts = destination.split(/[\\/]/).filter(Boolean);
  if (parts.length < 2) return destination;
  return `${parts[parts.length - 2]}\\${parts[parts.length - 1]}`;
}

interface FileTableProps {
  files: FileEntry[];
  selected: Set<string>;
  onToggle: (path: string) => void;
  onToggleAll: () => void;
  targetFolder: string;
  busy: boolean;
}

export function FileTable({
  files,
  selected,
  onToggle,
  onToggleAll,
  targetFolder,
  busy,
}: FileTableProps) {
  const allSelected = files.length > 0 && files.every((file) => selected.has(file.path));

  if (files.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center px-6 py-20 text-center">
        <span className="flex h-12 w-12 items-center justify-center rounded-xl bg-white/[0.04] text-slate-500">
          <FileQuestion className="h-6 w-6" aria-hidden />
        </span>
        <div className="mt-3 text-sm font-medium text-slate-300">
          Nothing to show here
        </div>
        <div className="mt-1 max-w-sm text-xs text-slate-500">
          {targetFolder
            ? "No top-level files match the current filter. Try another category or rescan the folder."
            : "Choose a target folder to preview the files that would be organized."}
        </div>
      </div>
    );
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full min-w-[860px] border-collapse text-left">
        <thead>
          <tr className="border-b border-white/[0.07] text-[11px] font-semibold uppercase tracking-wider text-slate-500">
            <th className="w-10 px-4 py-3">
              <input
                type="checkbox"
                className="checkbox"
                checked={allSelected}
                onChange={onToggleAll}
                disabled={busy}
                aria-label="Select all visible files"
              />
            </th>
            <th className="px-2 py-3 font-semibold">File</th>
            <th className="px-3 py-3 font-semibold">Category</th>
            <th className="px-3 py-3 text-right font-semibold">Size</th>
            <th className="px-3 py-3 font-semibold">Modified</th>
            <th className="px-3 py-3 font-semibold">Status</th>
            <th className="px-4 py-3 font-semibold">Destination</th>
          </tr>
        </thead>
        <tbody>
          {files.map((file) => {
            const isSelected = selected.has(file.path);
            return (
              <tr
                key={file.path}
                className={`border-b border-white/[0.04] transition-colors last:border-0 ${
                  isSelected ? "bg-sky-500/[0.04]" : "hover:bg-white/[0.02]"
                }`}
              >
                <td className="px-4 py-3">
                  <input
                    type="checkbox"
                    className="checkbox"
                    checked={isSelected}
                    onChange={() => onToggle(file.path)}
                    disabled={busy}
                    aria-label={`Select ${file.name}`}
                  />
                </td>
                <td className="max-w-[240px] px-2 py-3">
                  <div className="flex items-center gap-2">
                    <Layers3 className="h-4 w-4 shrink-0 text-slate-600" aria-hidden />
                    <span
                      className="truncate text-[13px] font-medium text-slate-200"
                      title={file.name}
                    >
                      {file.name}
                    </span>
                  </div>
                </td>
                <td className="px-3 py-3">
                  <CategoryBadge category={file.category} />
                </td>
                <td className="whitespace-nowrap px-3 py-3 text-right text-[13px] tabular-nums text-slate-400">
                  {formatSize(file.size)}
                </td>
                <td className="whitespace-nowrap px-3 py-3 text-[13px] text-slate-400">
                  {formatDate(file.modified)}
                </td>
                <td className="px-3 py-3">
                  {file.isDuplicate ? (
                    <span
                      className="inline-flex items-center gap-1 rounded-md bg-amber-500/10 px-2 py-0.5 text-[11px] font-medium text-amber-300 ring-1 ring-inset ring-amber-500/25"
                      title={`Duplicate group ${file.duplicateGroup ?? ""}`}
                    >
                      <Layers3 className="h-3 w-3" aria-hidden />
                      Duplicate
                    </span>
                  ) : (
                    <span className="text-[11px] text-slate-600">—</span>
                  )}
                </td>
                <td className="max-w-[280px] px-4 py-3">
                  <div className="flex items-center gap-1.5">
                    <FolderInput className="h-3.5 w-3.5 shrink-0 text-slate-600" aria-hidden />
                    <span
                      className="truncate font-mono text-[11px] text-slate-400"
                      title={file.destination}
                    >
                      {shortDestination(file.destination)}
                    </span>
                  </div>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
