// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { EvidenceManifestPage } from "./EvidenceManifestPage";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

afterEach(cleanup);
beforeEach(() => invokeMock.mockReset());

const manifest = {
  manifestId: "f10b468a-5b3b-4a4d-9205-899427870681",
  format: "moonships.evidence-manifest" as const,
  schemaVersion: 1 as const,
  createdAt: "2026-10-26T12:00:00Z",
  assurance: "hash_integrity_only" as const,
  integrityNotice:
    "SHA-256 integrity metadata only; does not establish chain of custody or forensic validity.",
  entries: [
    {
      id: "b72f34e5-57c3-4fea-a09f-27faf6ab25ae",
      fileName: "capture.bin",
      sizeBytes: 3,
      sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    },
  ],
  manifestSha256: "a".repeat(64),
};

describe("M6-011 evidence manifest", () => {
  it("hashes only after explicit selection and saves only after a second action", async () => {
    invokeMock.mockImplementation((command?: string) => {
      if (!command) return Promise.resolve(null);
      if (command === "select_evidence_files") return Promise.resolve(manifest);
      if (command === "save_evidence_manifest") return Promise.resolve("evidence.json");
      return Promise.reject(new Error(`Unexpected command ${command}`));
    });
    render(<EvidenceManifestPage />);
    expect(invokeMock).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Choose evidence files and hash" }));
    expect(await screen.findByText("capture.bin")).toBeTruthy();
    expect(
      (await screen.findAllByText(/does not establish chain of custody/)).length,
    ).toBeGreaterThan(0);
    expect(invokeMock).toHaveBeenCalledWith("select_evidence_files");
    fireEvent.click(screen.getByRole("button", { name: "Save manifest JSON" }));
    expect(await screen.findByText("Manifest saved as evidence.json.")).toBeTruthy();
    expect(invokeMock).toHaveBeenCalledWith("save_evidence_manifest", {
      request: { manifestId: manifest.manifestId },
    });
  });

  it("reports cancelled selection separately from an unavailable native picker", async () => {
    invokeMock.mockResolvedValueOnce(null).mockRejectedValueOnce(new Error("unavailable"));
    render(<EvidenceManifestPage />);
    fireEvent.click(screen.getByRole("button", { name: "Choose evidence files and hash" }));
    expect(
      await screen.findByText("File selection cancelled. No manifest was created."),
    ).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Choose evidence files and hash" }));
    expect(await screen.findByRole("alert")).toBeTruthy();
  });
});
