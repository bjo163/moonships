// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { IntelligenceSourcesPage } from "./IntelligenceSourcesPage";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

afterEach(cleanup);
beforeEach(() => invokeMock.mockReset());

describe("M5-008 Intelligence Sources", () => {
  it("loads saved settings without fetching and shows the empty state", async () => {
    invokeMock.mockResolvedValue([]);
    render(<IntelligenceSourcesPage />);

    expect(await screen.findByText("No sources configured")).toBeTruthy();
    expect(invokeMock.mock.calls.map(([command]) => command)).toEqual([
      "list_intelligence_sources",
    ]);
    expect(screen.getByRole("note").textContent).toMatch(/does not schedule background fetches/i);
  });

  it("does not turn a storage error into a false empty state and can retry", async () => {
    invokeMock.mockRejectedValueOnce(new Error("offline")).mockResolvedValueOnce([]);
    render(<IntelligenceSourcesPage />);

    expect(await screen.findByText("Saved sources are unavailable")).toBeTruthy();
    expect(screen.queryByText("No sources configured")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Retry loading sources" }));
    expect(await screen.findByText("No sources configured")).toBeTruthy();
  });

  it("saves a source disabled by default and enables it only by explicit action", async () => {
    let sources: Array<{
      connector: {
        id: string;
        displayName: string;
        sourceUrl: string;
        termsUrl: string | null;
        termsNotes: string;
        termsReview: "unreviewed" | "reviewed" | "restricted";
        minimumPollIntervalSeconds: number;
        enabled: boolean;
        health: string;
      };
      freshness: { status: "disabled" | "unknown"; lastSuccessAt: string | null; isLive: boolean };
    }> = [];
    invokeMock.mockImplementation(
      async (command: string, args?: { request?: Record<string, unknown> }) => {
        if (command === "list_intelligence_sources") return sources;
        if (command === "save_intelligence_source") {
          const request = args?.request ?? {};
          const connector = {
            id: "rss-example",
            displayName: request.displayName as string,
            sourceUrl: request.sourceUrl as string,
            termsUrl: request.termsUrl as string | null,
            termsNotes: request.termsNotes as string,
            termsReview: request.termsReview as "unreviewed" | "reviewed" | "restricted",
            minimumPollIntervalSeconds: request.minimumPollIntervalSeconds as number,
            enabled: false,
            health: "not_configured",
          };
          sources = [
            { connector, freshness: { status: "disabled", lastSuccessAt: null, isLive: false } },
          ];
          return sources[0];
        }
        if (command === "set_intelligence_source_enabled") {
          sources = sources.map((source) => ({
            connector: {
              ...source.connector,
              enabled: Boolean(args?.request?.enabled),
              health: "ready",
            },
            freshness: { ...source.freshness, status: "unknown", isLive: false },
          }));
          return sources[0];
        }
        if (command === undefined) return sources;
        throw new Error(`Unexpected command: ${String(command)} args=${JSON.stringify(args)}`);
      },
    );

    render(<IntelligenceSourcesPage />);
    await screen.findByText("No sources configured");
    fireEvent.change(screen.getByLabelText("Display name"), { target: { value: "Example RSS" } });
    fireEvent.change(screen.getByLabelText("RSS or Atom feed URL"), {
      target: { value: "https://example.test/feed.xml" },
    });
    fireEvent.change(screen.getByLabelText("Terms and access notes"), {
      target: { value: "Reviewed feed terms; respect published limits." },
    });
    fireEvent.change(screen.getByLabelText("Terms review"), { target: { value: "reviewed" } });
    fireEvent.click(screen.getByRole("button", { name: "Save source" }));

    const card = await screen.findByRole("article");
    expect(within(card).getAllByText("Disabled")).toHaveLength(2);
    expect(within(card).getByText("Every 15 minutes")).toBeTruthy();
    expect(await screen.findByText("Source settings saved locally.")).toBeTruthy();
    expect(invokeMock.mock.calls.some(([command]) => command === "fetch_rss")).toBe(false);

    fireEvent.click(within(card).getByRole("button", { name: "Enable" }));
    await waitFor(() => expect(within(card).getByText("Enabled for manual fetch")).toBeTruthy());
    expect(invokeMock.mock.calls.some(([command]) => command === "fetch_rss")).toBe(false);
    expect(invokeMock.mock.calls.filter(([command]) => command === undefined)).toHaveLength(0);
  });

  it("adds an unreviewed NWS Atom alert preset disabled and without fetching", async () => {
    let savedRequest: Record<string, unknown> | null = null;
    const sources: Array<{
      connector: {
        id: string;
        displayName: string;
        sourceUrl: string;
        termsUrl: string | null;
        termsNotes: string;
        termsReview: "unreviewed" | "reviewed" | "restricted";
        minimumPollIntervalSeconds: number;
        enabled: boolean;
        health: string;
      };
      freshness: { status: "disabled" | "unknown"; lastSuccessAt: string | null; isLive: boolean };
    }> = [];
    invokeMock.mockImplementation(
      async (command: string, args?: { request?: Record<string, unknown> }) => {
        if (command === "list_intelligence_sources") return sources;
        if (command === "save_intelligence_source") {
          savedRequest = args?.request ?? null;
          sources.push({
            connector: {
              id: "rss-nws-ca",
              displayName: String(savedRequest?.displayName),
              sourceUrl: String(savedRequest?.sourceUrl),
              termsUrl: String(savedRequest?.termsUrl),
              termsNotes: String(savedRequest?.termsNotes),
              termsReview: "unreviewed",
              minimumPollIntervalSeconds: 1800,
              enabled: false,
              health: "not_configured",
            },
            freshness: { status: "disabled", lastSuccessAt: null, isLive: false },
          });
          return sources[0];
        }
        if (command === undefined) return sources;
        throw new Error(`Unexpected command: ${command}`);
      },
    );

    render(<IntelligenceSourcesPage />);
    await screen.findByText("No sources configured");
    const addButton = screen.getByRole("button", { name: "Add NWS alert feed" });
    fireEvent.click(addButton);
    expect((await screen.findByRole("alert")).textContent).toContain("Choose a state or district");
    expect(savedRequest).toBeNull();

    fireEvent.change(screen.getByLabelText("State or district code"), {
      target: { value: "CA" },
    });
    fireEvent.click(addButton);
    expect(await screen.findByText(/Added the NWS CA feed disabled/)).toBeTruthy();
    expect(savedRequest).toMatchObject({
      displayName: "NWS Active Alerts (CA)",
      sourceUrl: "https://api.weather.gov/alerts/active.atom?area=CA",
      termsUrl: "https://www.weather.gov/documentation/services-web-alerts",
      termsReview: "unreviewed",
      minimumPollIntervalSeconds: 1800,
    });
    expect(sources[0].connector.enabled).toBe(false);
    expect(invokeMock.mock.calls.some(([command]) => command === "fetch_intelligence_source")).toBe(
      false,
    );
  });
});
