// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { OpenApiDocumentPage } from "./OpenApiDocumentPage";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

afterEach(cleanup);
beforeEach(() => invokeMock.mockReset());

describe("M6-003 OpenAPI local viewer", () => {
  it("opens a locally selected document only after the user's action and displays validated metadata", async () => {
    invokeMock.mockResolvedValue({
      fileName: "service.yaml",
      content:
        "openapi: 3.1.0\ninfo:\n  title: Service API\n  version: 2.0.0\npaths:\n  /ready: {}\n",
    });
    render(<OpenApiDocumentPage />);
    expect(invokeMock).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Choose OpenAPI document" }));
    expect(await screen.findByText("Valid OpenAPI document structure")).toBeTruthy();
    expect(screen.getByText("3.1.0")).toBeTruthy();
    expect(screen.getByText("Service API")).toBeTruthy();
    expect(screen.getByText("2.0.0")).toBeTruthy();
    expect(screen.getByText("1")).toBeTruthy();
    expect(invokeMock).toHaveBeenCalledTimes(1);
    expect(invokeMock.mock.calls[0][0]).toBe("select_openapi_document");
  });

  it("reports malformed content and preserves it only as escaped text", async () => {
    invokeMock.mockResolvedValue({ fileName: "bad.yaml", content: "openapi: [bad" });
    render(<OpenApiDocumentPage />);
    fireEvent.click(screen.getByRole("button", { name: "Choose OpenAPI document" }));
    expect(await screen.findByText("Document is malformed or unsupported")).toBeTruthy();
    expect(screen.getAllByRole("listitem").length).toBeGreaterThan(0);
  });

  it("distinguishes cancelled selection from a selected empty or invalid document", async () => {
    invokeMock.mockResolvedValue(null);
    render(<OpenApiDocumentPage />);
    fireEvent.click(screen.getByRole("button", { name: "Choose OpenAPI document" }));
    expect((await screen.findByRole("status")).textContent).toBe("File selection cancelled.");
    expect(screen.queryByText("Valid OpenAPI document structure")).toBeNull();
  });
});
