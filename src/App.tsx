import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { FilterBar } from "./components/FilterBar";
import { FileTable } from "./components/FileTable";
import { SummaryCards } from "./components/SummaryCards";
import { Toast } from "./components/Toast";
import { TopBar } from "./components/TopBar";
import * as api from "./lib/api";
import { errorMessage } from "./lib/format";
import type {
  BatchInfo,
  BusyState,
  Category,
  Filter,
  ScanResult,
  Settings,
} from "./types";
import { CATEGORIES } from "./types";

type Notice = { kind: "ok" | "err"; text: string };

export default function App() {
  const [folder, setFolder] = useState("");
  const [scan, setScan] = useState<ScanResult | null>(null);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [filter, setFilter] = useState<Filter>("all");
  const [batchInfo, setBatchInfo] = useState<BatchInfo | null>(null);
  const [settings, setSettings] = useState<Settings | null>(null);
  const [busy, setBusy] = useState<BusyState>(null);
  const [notice, setNotice] = useState<Notice | null>(null);

  const notify = useCallback((kind: Notice["kind"], text: string) => {
    setNotice({ kind, text });
  }, []);

  useEffect(() => {
    if (!notice) return;
    const timer = window.setTimeout(() => setNotice(null), 5000);
    return () => window.clearTimeout(timer);
  }, [notice]);

  const folderRef = useRef("");
  useEffect(() => {
    folderRef.current = folder;
  }, [folder]);

  const refreshBatchInfo = useCallback(async () => {
    setBatchInfo(await api.getLastBatchInfo());
  }, []);

  const runScan = useCallback(
    async (target: string, resetFilter = false) => {
      setBusy("scan");
      try {
        const result = await api.scanFolder(target);
        setScan(result);
        // Everything is selected by default so the user can simply hit Organize.
        setSelected(new Set(result.files.map((file) => file.path)));
        if (resetFilter) setFilter("all");
      } catch (error) {
        notify("err", `Scan failed: ${errorMessage(error)}`);
      } finally {
        setBusy(null);
      }
    },
    [notify],
  );

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const defaultFolder = await api.defaultDownloadsDir();
        if (cancelled) return;
        setFolder(defaultFolder);
        await runScan(defaultFolder);
        if (!cancelled) await refreshBatchInfo();
        if (!cancelled) setSettings(await api.getSettings());
      } catch (error) {
        if (!cancelled) notify("err", errorMessage(error));
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [runScan, refreshBatchInfo, notify]);

  // Background auto-sorts and tray toggles notify the UI through Tauri events.
  useEffect(() => {
    let disposed = false;
    const unlisteners: Array<() => void> = [];

    listen<string>("droptidy-auto-sorted", (event) => {
      notify("ok", `Auto-organized ${event.payload}`);
      void runScan(folderRef.current);
    })
      .then((unlisten) => (disposed ? unlisten() : unlisteners.push(unlisten)))
      .catch(() => {});

    listen<Settings>("droptidy-settings-changed", (event) => {
      setSettings(event.payload);
    })
      .then((unlisten) => (disposed ? unlisten() : unlisteners.push(unlisten)))
      .catch(() => {});

    return () => {
      disposed = true;
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, [runScan, notify]);

  const files = scan?.files ?? [];

  const visibleFiles = useMemo(() => {
    if (filter === "all") return files;
    if (filter === "duplicates") return files.filter((file) => file.isDuplicate);
    return files.filter((file) => file.category === filter);
  }, [files, filter]);

  const counts = useMemo(() => {
    const result: Record<string, number> = { all: files.length, duplicates: 0 };
    for (const category of CATEGORIES) result[category] = 0;
    for (const file of files) {
      result[file.category] = (result[file.category] ?? 0) + 1;
      if (file.isDuplicate) result.duplicates += 1;
    }
    return result;
  }, [files]);

  const categorySelection = useMemo(() => {
    const result = {} as Record<Category, { selected: number; total: number }>;
    for (const category of CATEGORIES) {
      const inCategory = files.filter((file) => file.category === category);
      result[category] = {
        selected: inCategory.filter((file) => selected.has(file.path)).length,
        total: inCategory.length,
      };
    }
    return result;
  }, [files, selected]);

  const toggleFile = useCallback((path: string) => {
    setSelected((previous) => {
      const next = new Set(previous);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  }, []);

  const toggleCategory = useCallback(
    (category: Category) => {
      setSelected((previous) => {
        const next = new Set(previous);
        const inCategory = files.filter((file) => file.category === category);
        const allSelected =
          inCategory.length > 0 && inCategory.every((file) => next.has(file.path));
        for (const file of inCategory) {
          if (allSelected) next.delete(file.path);
          else next.add(file.path);
        }
        return next;
      });
    },
    [files],
  );

  const toggleAllVisible = useCallback(() => {
    setSelected((previous) => {
      const next = new Set(previous);
      const allSelected =
        visibleFiles.length > 0 &&
        visibleFiles.every((file) => next.has(file.path));
      for (const file of visibleFiles) {
        if (allSelected) next.delete(file.path);
        else next.add(file.path);
      }
      return next;
    });
  }, [visibleFiles]);

  const handlePickFolder = useCallback(async () => {
    setBusy("pick");
    try {
      const picked = await api.pickFolder();
      if (picked) {
        setFolder(picked);
        // Keep the watcher pointed at the newly selected folder.
        if (settings?.autoOrganize) {
          setSettings(await api.setAutoOrganize(true, picked));
        }
        await runScan(picked, true);
      }
    } catch (error) {
      notify("err", `Folder picker failed: ${errorMessage(error)}`);
    } finally {
      setBusy(null);
    }
  }, [runScan, notify, settings]);

  const handleToggleAutoOrganize = useCallback(async () => {
    if (!settings) return;
    const next = !settings.autoOrganize;
    setBusy("settings");
    try {
      const updated = await api.setAutoOrganize(next, folder);
      setSettings(updated);
      notify(
        "ok",
        next
          ? "Auto-Organize enabled — new downloads will be sorted automatically."
          : "Auto-Organize disabled.",
      );
    } catch (error) {
      notify("err", `Could not update Auto-Organize: ${errorMessage(error)}`);
    } finally {
      setBusy(null);
    }
  }, [settings, folder, notify]);

  const handleOrganize = useCallback(async () => {
    const paths = [...selected];
    if (paths.length === 0) {
      notify("err", "Select at least one file to organize.");
      return;
    }
    setBusy("organize");
    try {
      const result = await api.organizeSelected(paths, folder);
      if (result.errors.length > 0) {
        notify(
          "err",
          `Moved ${result.moved} file(s), but ${result.errors.length} failed: ${result.errors[0]}`,
        );
      } else {
        notify("ok", `Organized ${result.moved} file(s) into category folders.`);
      }
      await refreshBatchInfo();
      await runScan(folder);
    } catch (error) {
      notify("err", `Organize failed: ${errorMessage(error)}`);
    } finally {
      setBusy(null);
    }
  }, [selected, folder, notify, refreshBatchInfo, runScan]);

  const handleUndo = useCallback(async () => {
    setBusy("undo");
    try {
      const result = await api.undoLastBatch();
      if (result.errors.length > 0) {
        notify(
          "err",
          `Restored ${result.restored} file(s), but ${result.errors.length} failed: ${result.errors[0]}`,
        );
      } else {
        notify("ok", `Restored ${result.restored} file(s) to their original locations.`);
      }
      await refreshBatchInfo();
      await runScan(folder);
    } catch (error) {
      notify("err", `Undo failed: ${errorMessage(error)}`);
    } finally {
      setBusy(null);
    }
  }, [folder, notify, refreshBatchInfo, runScan]);

  return (
    <div className="min-h-screen bg-[#0f1117] text-slate-200">
      <TopBar
        folder={folder}
        selectedCount={selected.size}
        batchInfo={batchInfo}
        autoOrganize={settings?.autoOrganize ?? false}
        busy={busy}
        onPickFolder={handlePickFolder}
        onRescan={() => void runScan(folder)}
        onToggleAutoOrganize={() => void handleToggleAutoOrganize()}
        onUndo={() => void handleUndo()}
        onOrganize={() => void handleOrganize()}
      />

      <main className="mx-auto max-w-[1220px] space-y-6 px-6 py-6">
        <SummaryCards
          totalFiles={files.length}
          totalSize={scan?.totalSize ?? 0}
          duplicateCount={scan?.duplicateCount ?? 0}
        />

        <section className="panel overflow-hidden">
          <div className="flex flex-wrap items-center justify-between gap-3 border-b border-white/[0.07] px-5 py-4">
            <div>
              <h2 className="text-sm font-semibold text-white">Dry-run preview</h2>
              <p className="mt-0.5 text-xs text-slate-500">
                Review what will move before you organize. Sub-folders are never scanned.
              </p>
            </div>
            <span className="rounded-md bg-white/[0.04] px-2.5 py-1 text-xs tabular-nums text-slate-400 ring-1 ring-inset ring-white/[0.06]">
              {visibleFiles.length} shown · {selected.size} selected
            </span>
          </div>

          <div className="px-5 py-4">
            <FilterBar
              filter={filter}
              onFilterChange={setFilter}
              counts={counts}
              categorySelection={categorySelection}
              onToggleCategory={toggleCategory}
            />
          </div>

          <FileTable
            files={visibleFiles}
            selected={selected}
            onToggle={toggleFile}
            onToggleAll={toggleAllVisible}
            targetFolder={folder}
            busy={busy !== null}
          />
        </section>

        <footer className="pb-2 text-center text-[11px] text-slate-600">
          DropTidy · Files are only moved inside your chosen folder · Every sort can be undone
        </footer>
      </main>

      <Toast toast={notice} onClose={() => setNotice(null)} />
    </div>
  );
}
