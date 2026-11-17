import { Copy, FileStack, HardDrive } from "lucide-react";
import type { ReactNode } from "react";
import { formatSize } from "../lib/format";

interface SummaryCardsProps {
  totalFiles: number;
  totalSize: number;
  duplicateCount: number;
}

function Card({
  icon,
  label,
  value,
  hint,
  accent,
}: {
  icon: ReactNode;
  label: string;
  value: string;
  hint: string;
  accent: string;
}) {
  return (
    <div className="panel flex items-start gap-4 p-5">
      <span
        className={`flex h-11 w-11 shrink-0 items-center justify-center rounded-lg ring-1 ring-inset ${accent}`}
      >
        {icon}
      </span>
      <div className="min-w-0">
        <div className="text-xs font-medium uppercase tracking-wider text-slate-500">
          {label}
        </div>
        <div className="mt-1 text-2xl font-semibold tabular-nums tracking-tight text-white">
          {value}
        </div>
        <div className="mt-0.5 truncate text-xs text-slate-500">{hint}</div>
      </div>
    </div>
  );
}

export function SummaryCards({
  totalFiles,
  totalSize,
  duplicateCount,
}: SummaryCardsProps) {
  return (
    <section className="grid gap-4 sm:grid-cols-3">
      <Card
        icon={<FileStack className="h-5 w-5" aria-hidden />}
        label="Files found"
        value={totalFiles.toLocaleString()}
        hint="Top-level files in the target folder"
        accent="bg-sky-500/10 text-sky-400 ring-sky-400/20"
      />
      <Card
        icon={<HardDrive className="h-5 w-5" aria-hidden />}
        label="Sortable size"
        value={formatSize(totalSize)}
        hint="Space held by sortable downloads"
        accent="bg-emerald-500/10 text-emerald-400 ring-emerald-400/20"
      />
      <Card
        icon={<Copy className="h-5 w-5" aria-hidden />}
        label="Duplicates"
        value={duplicateCount.toLocaleString()}
        hint={
          duplicateCount > 0
            ? "Identical copies detected by SHA-256"
            : "No identical copies detected"
        }
        accent="bg-amber-500/10 text-amber-400 ring-amber-400/20"
      />
    </section>
  );
}
