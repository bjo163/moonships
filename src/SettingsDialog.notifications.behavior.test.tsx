// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { SettingsDialog } from "./App";

const { invokeMock, isPermissionGrantedMock, requestPermissionMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
  isPermissionGrantedMock: vi.fn(),
  requestPermissionMock: vi.fn(),
}));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("@tauri-apps/plugin-notification", () => ({
  isPermissionGranted: isPermissionGrantedMock,
  requestPermission: requestPermissionMock,
}));

afterEach(cleanup);
beforeEach(() => {
  invokeMock
    .mockReset()
    .mockImplementation((command: string) =>
      Promise.resolve(
        command === "get_notification_preferences" ? { enabled: false } : { enabled: true },
      ),
    );
  isPermissionGrantedMock.mockReset().mockResolvedValue(false);
  requestPermissionMock.mockReset().mockResolvedValue("granted");
});

const renderSettings = () =>
  render(
    <SettingsDialog
      theme="dark"
      onToggleTheme={() => {}}
      onClose={() => {}}
      hideToTray={false}
      hideToTrayStatus="Preference loaded from this device."
      onToggleHideToTray={() => {}}
    />,
  );

describe("M5-014 explicit desktop notification preference", () => {
  it("requests OS permission from the user's enable action before persisting opt-in", async () => {
    renderSettings();
    const checkbox = await screen.findByRole("checkbox", {
      name: "Desktop notifications after manual Intelligence fetch",
    });
    await waitFor(() => expect((checkbox as HTMLInputElement).disabled).toBe(false));
    fireEvent.click(checkbox);
    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("set_notification_preferences", {
        request: { enabled: true },
      }),
    );
    expect(isPermissionGrantedMock).toHaveBeenCalledTimes(1);
    expect(requestPermissionMock).toHaveBeenCalledTimes(1);
    expect((checkbox as HTMLInputElement).checked).toBe(true);
    expect(screen.getByText(/only after a manual fetch/i)).toBeTruthy();
  });

  it("keeps the setting off when system permission is denied", async () => {
    requestPermissionMock.mockResolvedValue("denied");
    renderSettings();
    const checkbox = await screen.findByRole("checkbox", {
      name: "Desktop notifications after manual Intelligence fetch",
    });
    await waitFor(() => expect((checkbox as HTMLInputElement).disabled).toBe(false));
    fireEvent.click(checkbox);
    expect(await screen.findByText(/permission was not granted/i)).toBeTruthy();
    expect((checkbox as HTMLInputElement).checked).toBe(false);
    expect(invokeMock).not.toHaveBeenCalledWith("set_notification_preferences", {
      request: { enabled: true },
    });
  });

  it("does not overwrite a preference that could not be loaded", async () => {
    invokeMock.mockRejectedValue(new Error("preference unavailable"));
    renderSettings();
    const checkbox = await screen.findByRole("checkbox", {
      name: "Desktop notifications after manual Intelligence fetch",
    });
    await screen.findByText("Notification preferences require the Moonships desktop app.");
    expect((checkbox as HTMLInputElement).disabled).toBe(true);
    fireEvent.click(checkbox);
    expect(invokeMock).not.toHaveBeenCalledWith("set_notification_preferences", expect.anything());
  });
});
