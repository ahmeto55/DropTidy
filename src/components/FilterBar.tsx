import { Check, Copy, Layers3 } from "lucide-react";
import type { Category, Filter } from "../types";
import { CATEGORIES, CATEGORY_LABELS } from "../types";

interface FilterBarProps {
  filter: Filter;
  onFilterChange: (filter: Filter) => void;
  counts: Record<string, number>;
  categorySelection: Record<Category, { selected: number; total: number }>;
  onToggleCategory: (category: Category) => void;
}

function pillClass(active: boolean): string {
  return active
    ? "border-sky-400/30 bg-sky-500/15 text-sky-200"
    : "border-white/[0.08] bg-white/[0.03] text-slate-400 hover:border-white/[0.16] hover:text-slate-200";
}

export function FilterBar({
  filter,
  onFilterChange,
  counts,
  categorySelection,
  onToggleCategory,
}: FilterBarProps) {
  return (
    <div className="flex flex-wrap items-center gap-2">
      <span className="mr-1 text-[11px] font-semibold uppercase tracking-wider text-slate-500">
        Filter
      </span>

      <button
        type="button"
        onClick={() => onFilterChange("all")}
        className={`inline-flex items-center gap-1.5 rounded-lg border px-3 py-1.5 text-xs font-medium transition-colors ${pillClass(filter === "all")}`}
      >
        <Layers3 className="h-3.5 w-3.5" aria-hidden />
        All files
        <span className="tabular-nums opacity-60">{counts.all ?? 0}</span>
      </button>

      <button
        type="button"
        onClick={() => onFilterChange("duplicates")}
        className={`inline-flex items-center gap-1.5 rounded-lg border px-3 py-1.5 text-xs font-medium transition-colors ${pillClass(filter === "duplicates")}`}
      >
        <Copy className="h-3.5 w-3.5" aria-hidden />
        Duplicates only
        <span className="tabular-nums opacity-60">{counts.duplicates ?? 0}</span>
      </button>

      <span className="mx-1 h-5 w-px bg-white/10" aria-hidden />

      {CATEGORIES.map((category) => {
        const selection = categorySelection[category];
        const fullySelected = selection.total > 0 && selection.selected === selection.total;
        return (
          <div
            key={category}
            className={`inline-flex items-center gap-1.5 rounded-lg border pl-2 pr-2.5 py-1.5 transition-colors ${pillClass(filter === category)}`}
          >
            <label
              className="flex cursor-pointer items-center"
              title={`Toggle all ${CATEGORY_LABELS[category]} files`}
            >
              <input
                type="checkbox"
                className="checkbox h-3.5 w-3.5"
                checked={fullySelected}
                onChange={() => onToggleCategory(category)}
                aria-label={`Select all ${CATEGORY_LABELS[category]} files`}
              />
            </label>
            <button
              type="button"
              onClick={() => onFilterChange(category)}
              className="inline-flex items-center gap-1.5 text-xs font-medium"
            >
              {CATEGORY_LABELS[category]}
              <span className="tabular-nums opacity-60">
                {selection.selected}/{selection.total}
              </span>
              {fullySelected && <Check className="h-3 w-3 opacity-50" aria-hidden />}
            </button>
          </div>
        );
      })}
    </div>
  );
}
