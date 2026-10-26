// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { TopicWatchlistsPage } from "./TopicWatchlistsPage";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

afterEach(cleanup);
beforeEach(() => invokeMock.mockReset());

type Watchlist = {
  id: string;
  name: string;
  keywords: string[];
  categories: string[];
  paused: boolean;
};

describe("M5-009 Topic Watchlists", () => {
  it("loads an honest empty state without fetching or starting monitoring", async () => {
    invokeMock.mockResolvedValue([]);
    render(<TopicWatchlistsPage />);

    expect(await screen.findByText("No watchlists saved")).toBeTruthy();
    expect(screen.getByRole("note").textContent).toMatch(/do not start feed fetches/i);
    expect(invokeMock.mock.calls.map(([command]) => command)).toEqual([
      "list_intelligence_watchlists",
    ]);
  });

  it("saves local filters, pauses/resumes matching and confirms deletion", async () => {
    let watchlists: Watchlist[] = [];
    invokeMock.mockImplementation(
      async (command: string, args?: { request?: Record<string, unknown> }) => {
        if (command === "list_intelligence_watchlists") return watchlists;
        if (command === "save_intelligence_watchlist") {
          const request = args?.request ?? {};
          const watchlist: Watchlist = {
            id: "watch-test",
            name: request.name as string,
            keywords: request.keywords as string[],
            categories: request.categories as string[],
            paused: false,
          };
          watchlists = [watchlist];
          return watchlist;
        }
        if (command === "set_intelligence_watchlist_paused") {
          watchlists = watchlists.map((watchlist) => ({
            ...watchlist,
            paused: Boolean(args?.request?.paused),
          }));
          return watchlists[0];
        }
        if (command === "delete_intelligence_watchlist") {
          watchlists = [];
          return true;
        }
        if (command === undefined) return watchlists;
        throw new Error(`Unexpected command: ${command}`);
      },
    );
    render(<TopicWatchlistsPage />);
    await screen.findByText("No watchlists saved");
    fireEvent.change(screen.getByLabelText("Name"), { target: { value: "Space launches" } });
    fireEvent.change(screen.getByLabelText("Keywords, separated by commas"), {
      target: { value: "Artemis, NASA" },
    });
    fireEvent.change(screen.getByLabelText("Categories, separated by commas"), {
      target: { value: "general, advisory" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save watchlist" }));

    expect(screen.getByRole("link", { name: "Intelligence" }).getAttribute("href")).toBe(
      "#/intelligence",
    );

    const card = await screen.findByRole("article");
    expect(within(card).getByText("Artemis, NASA")).toBeTruthy();
    expect(within(card).getByText("general, advisory")).toBeTruthy();
    expect(invokeMock).toHaveBeenCalledWith("save_intelligence_watchlist", {
      request: {
        id: null,
        name: "Space launches",
        keywords: ["Artemis", "NASA"],
        categories: ["general", "advisory"],
      },
    });

    fireEvent.click(within(card).getByRole("button", { name: "Pause" }));
    expect(await within(card).findAllByText("Paused")).toHaveLength(2);
    expect(invokeMock).toHaveBeenCalledWith("set_intelligence_watchlist_paused", {
      request: { id: "watch-test", paused: true },
    });
    fireEvent.click(within(card).getByRole("button", { name: "Resume" }));
    expect(await within(card).findByText("Active filter")).toBeTruthy();

    fireEvent.click(within(card).getByRole("button", { name: "Delete" }));
    const dialog = screen.getByRole("dialog", { name: "Delete topic watchlist?" });
    fireEvent.click(within(dialog).getByRole("button", { name: "Delete watchlist" }));
    expect(await screen.findByText("No watchlists saved")).toBeTruthy();
    expect(invokeMock).toHaveBeenCalledWith("delete_intelligence_watchlist", {
      request: { id: "watch-test" },
    });
    expect(invokeMock.mock.calls.some(([command]) => command === "fetch_rss")).toBe(false);
  });

  it("distinguishes unavailable storage from the empty state and permits retry", async () => {
    invokeMock.mockRejectedValueOnce(new Error("offline")).mockResolvedValueOnce([]);
    render(<TopicWatchlistsPage />);
    expect(await screen.findByText("Saved watchlists are unavailable")).toBeTruthy();
    expect(screen.queryByText("No watchlists saved")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Retry loading watchlists" }));
    expect(await screen.findByText("No watchlists saved")).toBeTruthy();
  });
});
