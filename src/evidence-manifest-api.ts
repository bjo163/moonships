import { invoke } from "@tauri-apps/api/core";

export type EvidenceManifestEntry = {
  id: string;
  fileName: string;
  sizeBytes: number;
  sha256: string;
};

export type EvidenceManifest = {
  manifestId: string;
  format: "moonships.evidence-manifest";
  schemaVersion: 1;
  createdAt: string;
  assurance: "hash_integrity_only";
  integrityNotice: string;
  entries: EvidenceManifestEntry[];
  manifestSha256: string;
};

export function selectEvidenceFiles(): Promise<EvidenceManifest | null> {
  return invoke<EvidenceManifest | null>("select_evidence_files");
}

export function saveEvidenceManifest(manifestId: string): Promise<string | null> {
  return invoke<string | null>("save_evidence_manifest", { request: { manifestId } });
}
