// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { IntelligencePage } from "./IntelligencePage";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

const source = {
  connector: {
    id: "rss-jpl",
    displayName: "JPL Notices",
    sourceUrl: "https://example.test/feed",
    enabled: true,
    health: "healthy",
    lastAttemptAt: "2026-10-26T12:01:00Z",
  },
  freshness: { lastSuccessAt: "2026-10-26T12:00:00Z", isLive: true, status: "fresh" },
};
const item = {
  id: "item-1",
  sourceId: "rss-jpl",
  sourceUrl: "https://example.test/feed",
  originalUrl: "https://example.test/story",
  title: "Artemis launch update",
  summary: "Launch window confirmed",
  publishedAt: "2026-10-26T11:30:00Z",
  observedAt: "2026-10-26T12:00:00Z",
  eventTime: null,
  category: "general",
  freshnessState: "fresh",
  verificationState: "source_attributed",
};

afterEach(cleanup);
beforeEach(() => invokeMock.mockReset());

describe("M5-012 Intelligence list and item detail", () => {
  it("shows a real empty state separately from unavailable data and retries", async () => {
    invokeMock.mockImplementation((command: string) => {
      if (command === "list_intelligence_sources") return Promise.resolve([]);
      if (command === "list_intelligence_items") return Promise.resolve([]);
      return Promise.reject(new Error("unexpected"));
    });
    render(<IntelligencePage />);
    expect(await screen.findByText("No intelligence items saved")).toBeTruthy();

    cleanup();
    invokeMock.mockReset().mockRejectedValueOnce(new Error("storage unavailable"));
    render(<IntelligencePage />);
    expect(await screen.findByText("Saved intelligence items are unavailable")).toBeTruthy();
    invokeMock.mockResolvedValueOnce([]).mockResolvedValueOnce([item]);
    fireEvent.click(screen.getByRole("button", { name: "Retry" }));
    expect(await screen.findByText("Artemis launch update")).toBeTruthy();
  });

  it("filters saved items and shows source link plus separate published and observed timestamps", async () => {
    invokeMock.mockImplementation((command: string) =>
      Promise.resolve(
        command === "list_intelligence_sources"
          ? [source]
          : command === "list_intelligence_items"
            ? [item]
            : null,
      ),
    );
    render(<IntelligencePage />);
    expect(await screen.findByText("Artemis launch update")).toBeTruthy();
    expect(screen.getAllByText("JPL Notices").length).toBeGreaterThan(0);
    fireEvent.change(screen.getByRole("textbox", { name: "Search intelligence items" }), {
      target: { value: "no match" },
    });
    expect(screen.getByText("No items match these filters")).toBeTruthy();
    fireEvent.change(screen.getByRole("textbox", { name: "Search intelligence items" }), {
      target: { value: "artemis" },
    });
    fireEvent.click(screen.getByRole("link", { name: "Artemis launch update" }));
    window.location.hash = "#/intelligence/items/item-1";
    cleanup();
    invokeMock.mockImplementation((command: string) =>
      Promise.resolve(
        command === "list_intelligence_sources"
          ? [source]
          : command === "get_intelligence_item"
            ? item
            : null,
      ),
    );
    render(<IntelligencePage itemId="item-1" />);
    expect(await screen.findByText("Event time")).toBeTruthy();
    expect(screen.getByRole("link", { name: "Open original item" }).getAttribute("href")).toBe(
      item.originalUrl,
    );
    expect(screen.getByText("Published")).toBeTruthy();
    expect(screen.getByText("Observed")).toBeTruthy();
  });

  it("only fetches an explicitly enabled source and reports failures", async () => {
    invokeMock.mockImplementation((command: string) => {
      if (command === "list_intelligence_sources") return Promise.resolve([source]);
      if (command === "fetch_intelligence_source")
        return Promise.reject("FETCH_ERROR: RateLimited");
      if (command === "list_intelligence_items") return Promise.resolve([]);
      return Promise.resolve(null);
    });
    render(<IntelligencePage />);
    fireEvent.click(await screen.findByRole("button", { name: "Fetch now" }));
    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("fetch_intelligence_source", {
        request: { id: "rss-jpl" },
      }),
    );
    expect((await screen.findByRole("alert")).textContent).toContain("FETCH_ERROR: RateLimited");
    expect(screen.queryByText("No intelligence items saved")).toBeTruthy();
  });
});
