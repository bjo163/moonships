export type FindingSeverity = "informational" | "low" | "medium" | "high" | "critical";
export type FindingLikelihood = "rare" | "unlikely" | "possible" | "likely" | "almost_certain";

const SEVERITY_LABELS: Record<FindingSeverity, string> = {
  informational: "Informational",
  low: "Low",
  medium: "Medium",
  high: "High",
  critical: "Critical",
};

const LIKELIHOOD_LABELS: Record<FindingLikelihood, string> = {
  rare: "Rare",
  unlikely: "Unlikely",
  possible: "Possible",
  likely: "Likely",
  almost_certain: "Almost certain",
};

/** Reusable display keeps impact severity and likelihood as separate dimensions. */
export function SecurityFindingRiskFields({
  severity,
  likelihood,
}: {
  severity: FindingSeverity;
  likelihood: FindingLikelihood;
}) {
  return (
    <dl className="intelligence-metadata" aria-label="Finding severity and likelihood">
      <div>
        <dt>Severity</dt>
        <dd>{SEVERITY_LABELS[severity]}</dd>
      </div>
      <div>
        <dt>Likelihood</dt>
        <dd>{LIKELIHOOD_LABELS[likelihood]}</dd>
      </div>
    </dl>
  );
}
