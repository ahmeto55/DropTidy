import { CATEGORY_LABELS, type Category } from "../types";

const CATEGORY_STYLES: Record<Category, string> = {
  installers: "bg-rose-500/10 text-rose-300 ring-rose-500/25",
  archives: "bg-amber-500/10 text-amber-300 ring-amber-500/25",
  documents: "bg-sky-500/10 text-sky-300 ring-sky-500/25",
  images: "bg-emerald-500/10 text-emerald-300 ring-emerald-500/25",
  media: "bg-fuchsia-500/10 text-fuchsia-300 ring-fuchsia-500/25",
  code: "bg-indigo-500/10 text-indigo-300 ring-indigo-500/25",
  other: "bg-slate-500/10 text-slate-300 ring-slate-500/25",
};

export function CategoryBadge({ category }: { category: Category }) {
  return (
    <span
      className={`inline-flex items-center rounded-md px-2 py-0.5 text-[11px] font-medium ring-1 ring-inset ${CATEGORY_STYLES[category]}`}
    >
      {CATEGORY_LABELS[category]}
    </span>
  );
}
