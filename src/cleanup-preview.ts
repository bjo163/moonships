export type CleanupCategory = "temporary_files" | "application_cache";

export type CleanupPreviewDisposition = "eligible" | "skipped";

export type CleanupSkipReason =
  | "invalid_rule"
  | "wrong_root"
  | "not_regular_file"
  | "modified_time_in_future"
  | "too_recent"
  | "item_too_large";

export type CleanupPreviewItem = {
  candidateId: string;
  category: CleanupCategory;
  displayName: string;
  sizeBytes: number;
  modifiedAt: string;
  disposition: CleanupPreviewDisposition;
  skipReason: CleanupSkipReason | null;
};

export type CleanupPreviewDto = {
  generatedAt: string;
  items: CleanupPreviewItem[];
};
