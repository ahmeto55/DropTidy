export type Category =
  | "installers"
  | "archives"
  | "documents"
  | "images"
  | "media"
  | "code"
  | "other";

export type Filter = "all" | "duplicates" | Category;

export type BusyState = "scan" | "organize" | "undo" | "pick" | "settings" | null;

export interface FileEntry {
  path: string;
  name: string;
  extension: string;
  category: Category;
  size: number;
  modified: number;
  isDuplicate: boolean;
  duplicateGroup: string | null;
  destination: string;
}

export interface ScanResult {
  targetFolder: string;
  files: FileEntry[];
  totalSize: number;
  duplicateCount: number;
}

export interface OrganizeResult {
  batchId: string;
  moved: number;
  errors: string[];
}

export interface UndoResult {
  restored: number;
  errors: string[];
}

export interface BatchInfo {
  batchId: string;
  fileCount: number;
  timestamp: number;
}

export interface Settings {
  autoOrganize: boolean;
  watchDelaySecs: number;
}

export const CATEGORIES: Category[] = [
  "installers",
  "archives",
  "documents",
  "images",
  "media",
  "code",
  "other",
];

export const CATEGORY_LABELS: Record<Category, string> = {
  installers: "Installers",
  archives: "Archives",
  documents: "Documents",
  images: "Images",
  media: "Media",
  code: "Code",
  other: "Other",
};
