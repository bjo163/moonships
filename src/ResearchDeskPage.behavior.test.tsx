// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ResearchDeskPage } from "./ResearchDeskPage";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
afterEach(cleanup);
beforeEach(() => invokeMock.mockReset());

describe("M5-013 Research Desk provenance", () => {
  it("shows source attribution and retention state on the note detail and exports explicitly", async () => {
    invokeMock.mockImplementation((command: string) =>
      Promise.resolve(
        command === "list_research_notes"
          ? [
              {
                id: "note-1",
                title: "Artemis update",
                body: "",
                itemId: "item-1",
                createdAt: "2026-10-26T12:00:00Z",
                updatedAt: "2026-10-26T12:00:00Z",
              },
            ]
          : command === "get_research_note"
            ? {
                schemaVersion: 1,
                note: {
                  id: "note-1",
                  title: "Artemis update",
                  body: "",
                  itemId: "item-1",
                  createdAt: "2026-10-26T12:00:00Z",
                  updatedAt: "2026-10-26T12:00:00Z",
                },
                reference: {
                  itemId: "item-1",
                  sourceId: "rss-jpl",
                  sourceUrl: "https://example.test/feed",
                  originalUrl: "https://example.test/item",
                  publishedAt: null,
                  observedAt: "2026-10-26T12:00:00Z",
                  verificationState: "sourceattributed",
                  itemAvailable: false,
                },
              }
            : command === "export_research_note"
              ? "research.json"
              : null,
      ),
    );
    render(<ResearchDeskPage noteId="note-1" />);
    expect(await screen.findByText("rss-jpl")).toBeTruthy();
    expect(screen.getByText("Item unavailable; provenance snapshot retained")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Export note with provenance" }));
    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("export_research_note", {
        request: { id: "note-1" },
      }),
    );
  });

  it("distinguishes a real empty list from a failed note read", async () => {
    invokeMock.mockImplementation((command: string) => {
      if (command === "list_research_notes") return Promise.resolve([]);
      return Promise.resolve(null);
    });
    render(<ResearchDeskPage />);
    expect(await screen.findByText("No research notes saved")).toBeTruthy();
    cleanup();
    invokeMock
      .mockReset()
      .mockImplementation((command: string) =>
        Promise.resolve(command === "list_research_notes" ? null : null),
      );
    render(<ResearchDeskPage />);
    expect(await screen.findByText("Research notes are unavailable")).toBeTruthy();
  });
});
