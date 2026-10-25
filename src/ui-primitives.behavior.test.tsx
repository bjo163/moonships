// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useState } from "react";
import {
  Badge,
  Button,
  Dialog,
  EmptyState,
  ErrorState,
  Input,
  LoadingState,
  Progress,
  Toast,
} from "./ui-primitives";

afterEach(cleanup);

function DialogHarness() {
  const [open, setOpen] = useState(false);
  return (
    <>
      <button type="button" onClick={() => setOpen(true)}>
        Open preferences
      </button>
      <button type="button">Background action</button>
      <Dialog
        open={open}
        title="Preferences"
        description="Configure this workspace"
        onClose={() => setOpen(false)}
      >
        <p>Dialog content</p>
        <Input label="Nickname" />
        <Button variant="secondary">Apply</Button>
      </Dialog>
    </>
  );
}

describe("M1-041 DOM interaction and accessibility", () => {
  it("opens via the trigger, focuses inside, and restores the trigger on close", async () => {
    const user = userEvent.setup();
    render(<DialogHarness />);
    const trigger = screen.getByRole("button", { name: "Open preferences" });
    expect(screen.queryByRole("dialog")).toBeNull();

    await user.click(trigger);
    const dialog = screen.getByRole("dialog", { name: "Preferences" });
    expect(dialog.contains(document.activeElement)).toBe(true);
    expect(document.activeElement).toBe(
      within(dialog).getByRole("button", { name: "Close dialog" }),
    );
    expect(dialog.getAttribute("aria-describedby")).toBe(
      within(dialog).getByText("Configure this workspace").id,
    );

    await user.click(within(dialog).getByRole("button", { name: "Close dialog" }));
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(document.activeElement).toBe(trigger);
  });

  it("closes on Escape and returns focus to its trigger", async () => {
    const user = userEvent.setup();
    render(<DialogHarness />);
    const trigger = screen.getByRole("button", { name: "Open preferences" });
    await user.click(trigger);
    expect(screen.getByRole("dialog")).toBeTruthy();

    await user.keyboard("{Escape}");
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(document.activeElement).toBe(trigger);
  });

  it("wraps Tab and Shift+Tab within the modal instead of reaching background actions", async () => {
    const user = userEvent.setup();
    render(<DialogHarness />);
    await user.click(screen.getByRole("button", { name: "Open preferences" }));
    const dialog = screen.getByRole("dialog");
    const close = within(dialog).getByRole("button", { name: "Close dialog" });
    const apply = within(dialog).getByRole("button", { name: "Apply" });
    const focusable = Array.from(
      dialog.querySelectorAll<HTMLElement>("button:not(:disabled), input:not(:disabled)"),
    );
    expect(focusable).toEqual([
      close,
      within(dialog).getByRole("textbox", { name: "Nickname" }),
      apply,
    ]);
    expect(document.activeElement).toBe(close);

    // Test the browser's cancellable boundary event directly: jsdom userEvent.tab()
    // computes another synthetic focus move even after our handler changes focus.
    expect(fireEvent.keyDown(close, { key: "Tab", shiftKey: true, cancelable: true })).toBe(false);
    expect(document.activeElement).toBe(apply);
    expect(fireEvent.keyDown(apply, { key: "Tab", cancelable: true })).toBe(false);
    expect(document.activeElement).toBe(close);
    await user.tab();
    expect(document.activeElement).toBe(within(dialog).getByRole("textbox", { name: "Nickname" }));
    expect(dialog.contains(document.activeElement)).toBe(true);
  });

  it("keeps focus in the modal when the parent rerenders or outside content is focused", async () => {
    const user = userEvent.setup();
    const { rerender } = render(<DialogHarness />);
    await user.click(screen.getByRole("button", { name: "Open preferences" }));
    const dialog = screen.getByRole("dialog");
    const nickname = within(dialog).getByRole("textbox", { name: "Nickname" });
    nickname.focus();

    rerender(<DialogHarness />);
    expect(document.activeElement).toBe(nickname);
    screen.getByRole("button", { name: "Background action" }).focus();
    expect(dialog.contains(document.activeElement)).toBe(true);
  });

  it("dismisses via backdrop, but interaction inside the dialog does not dismiss", async () => {
    const user = userEvent.setup();
    const { container } = render(<DialogHarness />);
    const trigger = screen.getByRole("button", { name: "Open preferences" });
    await user.click(trigger);

    await user.click(within(screen.getByRole("dialog")).getByText("Dialog content"));
    expect(screen.getByRole("dialog")).toBeTruthy();

    const backdropButton = container.querySelector<HTMLButtonElement>(
      ".ms-dialog-backdrop-dismiss",
    );
    expect(backdropButton).not.toBeNull();
    await user.click(backdropButton as HTMLButtonElement);
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(document.activeElement).toBe(trigger);
  });

  it("associates input labels, hints, external descriptions, and validation errors", async () => {
    const user = userEvent.setup();
    const { rerender } = render(
      <>
        <p id="external-description">Public project name</p>
        <Input
          label="Project name"
          hint="Use a short title"
          aria-describedby="external-description"
        />
      </>,
    );
    const input = screen.getByRole("textbox", { name: "Project name" });
    expect(input.id).not.toBe("");
    expect(input.getAttribute("aria-describedby")).toContain("external-description");
    const hint = screen.getByText("Use a short title");
    expect(input.getAttribute("aria-describedby")?.split(" ")).toContain(hint.id);
    await user.type(input, "New project");
    expect((input as HTMLInputElement).value).toBe("New project");

    rerender(
      <>
        <p id="external-description">Public project name</p>
        <Input
          label="Project name"
          hint="Use a short title"
          error="Name is required"
          aria-describedby="external-description"
        />
      </>,
    );
    const invalid = screen.getByRole("textbox", { name: "Project name" });
    const error = screen.getByRole("alert");
    expect(invalid.getAttribute("aria-invalid")).toBe("true");
    expect(invalid.getAttribute("aria-describedby")?.split(" ")).toContain(error.id);
    expect(screen.queryByText("Use a short title")).toBeNull();
    expect((invalid as HTMLInputElement).value).toBe("New project");
  });

  it("prevents actions for disabled and loading buttons, while enabled buttons work", async () => {
    const user = userEvent.setup();
    const onClick = vi.fn();
    render(
      <>
        <Button disabled onClick={onClick}>
          Unavailable
        </Button>
        <Button loading loadingLabel="Saving" onClick={onClick}>
          Save
        </Button>
        <Button onClick={onClick}>Continue</Button>
      </>,
    );
    const disabled = screen.getByRole("button", { name: "Unavailable" }) as HTMLButtonElement;
    const loading = screen.getByRole("button", { name: "Saving" }) as HTMLButtonElement;
    expect(disabled.disabled).toBe(true);
    expect(loading.disabled).toBe(true);
    expect(loading.getAttribute("aria-busy")).toBe("true");
    await user.click(disabled);
    await user.click(loading);
    expect(onClick).not.toHaveBeenCalled();
    await user.click(screen.getByRole("button", { name: "Continue" }));
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("exposes polite and assertive statuses plus a working error retry action", async () => {
    const user = userEvent.setup();
    const onRetry = vi.fn();
    const { rerender } = render(<Toast title="Saved" message="Saved locally" />);
    const status = screen.getByRole("status");
    expect(status.getAttribute("aria-live")).toBe("polite");
    expect(status.textContent).toContain("Saved locally");

    rerender(<Toast title="Failed" tone="error" />);
    expect(screen.getByRole("alert").getAttribute("aria-live")).toBe("assertive");

    rerender(<LoadingState label="Loading projects" />);
    expect(screen.getByRole("status").textContent).toContain("Loading projects");

    rerender(<ErrorState description="Offline" onRetry={onRetry} />);
    expect(screen.getByRole("alert").textContent).toContain("Offline");
    await user.click(screen.getByRole("button", { name: "Try again" }));
    expect(onRetry).toHaveBeenCalledTimes(1);
  });

  it("bounds accessible native progress values and renders honest empty and badge states", () => {
    const { rerender } = render(<Progress label="Upload" value={180} />);
    const progress = screen.getByRole("progressbar", { name: "Upload" }) as HTMLProgressElement;
    expect(progress.value).toBe(100);
    expect(progress.max).toBe(100);
    rerender(<Progress label="Upload" value={-5} />);
    expect(progress.value).toBe(0);
    rerender(<Progress label="Upload" value={Number.POSITIVE_INFINITY} />);
    expect(progress.value).toBe(0);

    rerender(
      <>
        <EmptyState title="No projects" description="Create your first project" />
        <Badge variant="warning">Review required</Badge>
      </>,
    );
    expect(screen.getByText("No projects")).toBeTruthy();
    expect(screen.getByText("Review required")).toBeTruthy();
  });
});
