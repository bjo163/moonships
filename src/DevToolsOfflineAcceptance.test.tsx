// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { DevToolsPage } from "./DevToolsPage";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

beforeEach(() => {
  invokeMock.mockReset();
  invokeMock.mockImplementation(async (command: string) => {
    if (command === "hash_sha256_text") return "local-sha256-test-digest";
    if (command === "hash_sha256_selected_file") {
      return {
        cancelled: false,
        fileName: "offline.txt",
        bytesHashed: 5,
        sha256: "local-file-digest",
      };
    }
    throw new Error(`Unexpected native command: ${command}`);
  });
  vi.stubGlobal(
    "fetch",
    vi.fn(() => {
      throw new Error("Network is disabled in this acceptance test.");
    }),
  );
});

describe("M3-023 offline DevTools acceptance", () => {
  it("completes every local DevTool with fetch disabled and no provider/network calls", async () => {
    render(<DevToolsPage />);

    fireEvent.change(screen.getByLabelText("JSON input"), { target: { value: '{"local":true}' } });
    fireEvent.click(screen.getByRole("button", { name: "Validate & format" }));
    expect(screen.getByRole("status").textContent).toContain("Valid JSON");

    fireEvent.click(screen.getByRole("button", { name: "YAML formatter" }));
    fireEvent.change(screen.getByLabelText("YAML input"), { target: { value: "local: true\n" } });
    fireEvent.click(screen.getByRole("button", { name: "Validate & format" }));
    expect(screen.getByRole("status").textContent).toContain("Valid YAML");

    fireEvent.click(screen.getByRole("button", { name: "Base64" }));
    fireEvent.change(screen.getByLabelText("UTF-8 text input"), { target: { value: "local" } });
    fireEvent.click(screen.getByRole("button", { name: "Encode Base64" }));
    expect((screen.getByLabelText("Base64 output") as HTMLTextAreaElement).value).toBe("bG9jYWw=");

    fireEvent.click(screen.getByRole("button", { name: "URL" }));
    fireEvent.change(screen.getByLabelText("URL component input"), { target: { value: "a/b" } });
    fireEvent.click(screen.getByRole("button", { name: "Encode URL" }));
    expect((screen.getByLabelText("URL output") as HTMLTextAreaElement).value).toBe("a%2Fb");

    fireEvent.click(screen.getByRole("button", { name: "UUID" }));
    fireEvent.click(screen.getByRole("button", { name: "Generate UUID" }));
    expect(screen.getByLabelText("UUID v4").getAttribute("value")).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i,
    );

    fireEvent.click(screen.getByRole("button", { name: "SHA-256" }));
    fireEvent.change(screen.getByLabelText("Text input"), { target: { value: "local" } });
    fireEvent.click(screen.getByRole("button", { name: "Calculate SHA-256" }));
    expect(await screen.findByText("SHA-256 · local-sha256-test-digest")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Choose file and calculate SHA-256" }));
    expect(await screen.findByText(/offline\.txt · 5 bytes/)).toBeTruthy();

    fireEvent.click(screen.getByRole("button", { name: "Text diff" }));
    fireEvent.change(screen.getByLabelText("Original text"), { target: { value: "old\n" } });
    fireEvent.change(screen.getByLabelText("Updated text"), { target: { value: "new\n" } });
    fireEvent.click(screen.getByRole("button", { name: "Compare text" }));
    expect((screen.getByLabelText("Unified text diff") as HTMLTextAreaElement).value).toBe(
      "-old\n+new\n",
    );

    fireEvent.click(screen.getByRole("button", { name: "Timestamp" }));
    fireEvent.change(screen.getByLabelText("Unix timestamp input"), { target: { value: "0" } });
    fireEvent.click(screen.getByRole("button", { name: "Convert timestamp" }));
    expect(screen.getByText("1970-01-01T00:00:00.000Z")).toBeTruthy();

    await waitFor(() => expect(invokeMock).toHaveBeenCalledTimes(2));
    expect(invokeMock).toHaveBeenCalledWith("hash_sha256_text", { text: "local" });
    expect(invokeMock).toHaveBeenCalledWith("hash_sha256_selected_file");
    expect(fetch).not.toHaveBeenCalled();
  });
});
