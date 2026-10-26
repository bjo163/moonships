// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { Sha256DevToolsPanel } from "./Sha256DevToolsPanel";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

afterEach(cleanup);
beforeEach(() => invokeMock.mockReset());

describe("M3-020 SHA-256 DevTools", () => {
  it("sends text to the local native SHA-256 command and displays the digest", async () => {
    invokeMock.mockResolvedValue(
      "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    );
    render(<Sha256DevToolsPanel />);
    fireEvent.change(screen.getByLabelText("Text input"), { target: { value: "abc" } });
    fireEvent.click(screen.getByRole("button", { name: "Calculate SHA-256" }));

    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("hash_sha256_text", { text: "abc" }),
    );
    expect(await screen.findByText(/ba7816bf8f01cfea/)).toBeTruthy();
  });

  it("uses only the native file picker command and renders the returned leaf name and byte count", async () => {
    invokeMock.mockResolvedValue({
      cancelled: false,
      fileName: "notes.txt",
      bytesHashed: 3,
      sha256: "ba7816bf8f01cfea4",
    });
    render(<Sha256DevToolsPanel />);
    fireEvent.click(screen.getByRole("button", { name: "Choose file and calculate SHA-256" }));

    expect(await screen.findByText(/notes\.txt · 3 bytes/)).toBeTruthy();
    expect(invokeMock).toHaveBeenCalledWith("hash_sha256_selected_file");
    expect(invokeMock.mock.calls[0]).toHaveLength(1);
  });

  it("rejects oversized text before invoking native hashing", () => {
    render(<Sha256DevToolsPanel />);
    fireEvent.change(screen.getByLabelText("Text input"), {
      target: { value: "x".repeat(1024 * 1024 + 1) },
    });
    expect(screen.getByRole("alert").textContent).toContain("Text exceeds the 1 MiB limit.");
    expect(screen.getByRole("button", { name: "Calculate SHA-256" }).hasAttribute("disabled")).toBe(
      true,
    );
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("reports when the native file picker is cancelled", async () => {
    invokeMock.mockResolvedValue({ cancelled: true, fileName: null, bytesHashed: 0, sha256: null });
    render(<Sha256DevToolsPanel />);
    fireEvent.click(screen.getByRole("button", { name: "Choose file and calculate SHA-256" }));

    expect(await screen.findByText("File selection cancelled. No file was read.")).toBeTruthy();
  });
});
