// @vitest-environment jsdom
import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { SecurityFindingRiskFields } from "./SecurityFindingRiskFields";

describe("security finding risk dimensions", () => {
  it("renders severity and likelihood as separately labelled fields", () => {
    render(<SecurityFindingRiskFields severity="high" likelihood="possible" />);
    const fields = screen.getByLabelText("Finding severity and likelihood");
    expect(within(fields).getByText("Severity")).toBeTruthy();
    expect(within(fields).getByText("High")).toBeTruthy();
    expect(within(fields).getByText("Likelihood")).toBeTruthy();
    expect(within(fields).getByText("Possible")).toBeTruthy();
    expect(within(fields).queryByText("High / Possible")).toBeNull();
  });
});
