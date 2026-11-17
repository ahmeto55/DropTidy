import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  BatchInfo,
  OrganizeResult,
  ScanResult,
  Settings,
  UndoResult,
} from "../types";

export function defaultDownloadsDir(): Promise<string> {
  return invoke<string>("default_downloads_dir");
}

export function scanFolder(folder: string): Promise<ScanResult> {
  return invoke<ScanResult>("scan_folder", { folder });
}

export function organizeSelected(
  paths: string[],
  targetFolder: string,
): Promise<OrganizeResult> {
  return invoke<OrganizeResult>("organize_selected", {
    paths,
    targetFolder,
  });
}

export function undoLastBatch(): Promise<UndoResult> {
  return invoke<UndoResult>("undo_last_batch");
}

export function getLastBatchInfo(): Promise<BatchInfo | null> {
  return invoke<BatchInfo | null>("get_last_batch_info");
}

export function getSettings(): Promise<Settings> {
  return invoke<Settings>("get_settings");
}

export function setAutoOrganize(
  enabled: boolean,
  folder: string,
): Promise<Settings> {
  return invoke<Settings>("set_auto_organize", { enabled, folder });
}

export function setWatchDelaySecs(
  seconds: number,
  folder: string,
): Promise<Settings> {
  return invoke<Settings>("set_watch_delay_secs", { seconds, folder });
}

/** Let the user choose a custom target folder via the native folder picker. */
export async function pickFolder(): Promise<string | null> {
  const selection = await open({
    directory: true,
    multiple: false,
    title: "Choose a folder to organize",
  });
  return typeof selection === "string" ? selection : null;
}
