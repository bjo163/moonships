import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
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

describe("shared UI primitives", () => {
  it("renders a loading button as busy and disabled", () => {
    const markup = renderToStaticMarkup(
      <Button loading loadingLabel="Saving changes">
        Save
      </Button>,
    );

    expect(markup).toContain('aria-busy="true"');
    expect(markup).toContain("disabled");
    expect(markup).toContain("Saving changes");
    expect(markup).not.toContain(">Save</button>");
  });

  it("connects an invalid input to its accessible error message", () => {
    const markup = renderToStaticMarkup(
      <Input id="workspace-name" label="Workspace name" error="Name is required" />,
    );

    expect(markup).toContain('for="workspace-name"');
    expect(markup).toContain('aria-invalid="true"');
    expect(markup).toContain('aria-describedby="workspace-name-error"');
    expect(markup).toContain('role="alert"');
    expect(markup).toContain("Name is required");
  });

  it("generates a unique input ID when none is supplied and connects its hint", () => {
    const markup = renderToStaticMarkup(<Input label="Project code" hint="Optional code" />);
    const generatedId = markup.match(/id="(ms-input-[^"]+)"/)?.[1];

    expect(generatedId).toBeDefined();
    expect(markup).toContain(`for="${generatedId}"`);
    expect(markup).toContain(`aria-describedby="${generatedId}-hint"`);
    expect(markup).toContain(`id="${generatedId}-hint"`);
  });

  it("does not render a closed dialog and labels an open dialog", () => {
    const closed = renderToStaticMarkup(
      <Dialog open={false} title="Preferences" onClose={() => {}}>
        Body
      </Dialog>,
    );
    const open = renderToStaticMarkup(
      <Dialog open title="Preferences" description="Change local settings" onClose={() => {}}>
        Body
      </Dialog>,
    );

    expect(closed).toBe("");
    expect(open).toContain('role="dialog"');
    expect(open).toContain('aria-modal="true"');
    expect(open).toMatch(/aria-describedby="ms-dialog-description-[^"]+"/);
    expect(open).toContain("Change local settings");
  });

  it("provides explicit status semantics for toast, loading and errors", () => {
    expect(renderToStaticMarkup(<Toast title="Saved" />)).toContain('role="status"');
    expect(renderToStaticMarkup(<Toast title="Failed" tone="error" />)).toContain('role="alert"');
    expect(renderToStaticMarkup(<LoadingState label="Loading projects" />)).toContain(
      'aria-live="polite"',
    );
    expect(renderToStaticMarkup(<ErrorState description="Connection unavailable" />)).toContain(
      "Connection unavailable",
    );
    expect(
      renderToStaticMarkup(
        <EmptyState title="No projects" description="Create a project to begin." />,
      ),
    ).toContain("No projects");
  });

  it("uses semantic variants and bounds invalid progress values", () => {
    const markup = renderToStaticMarkup(
      <>
        <Badge variant="warning">Needs review</Badge>
        <Progress value={145} label="Import progress" />
      </>,
    );

    expect(markup).toContain("ms-badge--warning");
    expect(markup).toContain('value="100"');
    expect(markup).toContain("100%");
  });
});
