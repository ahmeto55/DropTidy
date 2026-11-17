import { CheckCircle2, X, XCircle } from "lucide-react";

interface ToastProps {
  toast: { kind: "ok" | "err"; text: string } | null;
  onClose: () => void;
}

export function Toast({ toast, onClose }: ToastProps) {
  if (!toast) return null;
  const success = toast.kind === "ok";

  return (
    <div
      role="status"
      className={`fixed bottom-6 right-6 z-50 flex max-w-md items-start gap-3 rounded-xl border px-4 py-3 text-sm shadow-2xl transition-all ${
        success
          ? "border-emerald-500/25 bg-[#12201c] text-emerald-200"
          : "border-rose-500/25 bg-[#211518] text-rose-200"
      }`}
    >
      {success ? (
        <CheckCircle2 className="mt-0.5 h-4 w-4 shrink-0 text-emerald-400" aria-hidden />
      ) : (
        <XCircle className="mt-0.5 h-4 w-4 shrink-0 text-rose-400" aria-hidden />
      )}
      <span className="flex-1 leading-snug">{toast.text}</span>
      <button
        type="button"
        onClick={onClose}
        className="shrink-0 rounded p-0.5 opacity-60 transition-opacity hover:opacity-100"
        aria-label="Dismiss notification"
      >
        <X className="h-4 w-4" aria-hidden />
      </button>
    </div>
  );
}
