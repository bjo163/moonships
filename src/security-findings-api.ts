import { invoke } from "@tauri-apps/api/core";
import type { FindingLikelihood, FindingSeverity } from "./SecurityFindingRiskFields";
import type { TaskRecord } from "./tasks-api";

export type AssessmentScopeTarget = {
  kind: "hostname" | "ip_address";
  value: string;
};

export type AssessmentScopeInput = {
  authorizedTargets: AssessmentScopeTarget[];
  authorizationNote: string;
  excludedTargets: AssessmentScopeTarget[];
  expiresAt: string;
};

export type SecurityFindingInput = {
  id: string;
  title: string;
  summary: string;
  severity: FindingSeverity;
  likelihood: FindingLikelihood;
  state: "open" | "accepted" | "mitigated" | "false_positive";
  target: AssessmentScopeTarget;
  observedAt: string;
};

export function createTaskFromSecurityFinding(request: {
  finding: SecurityFindingInput;
  scope: AssessmentScopeInput;
  remediation: string;
  evidenceUrl: string;
}): Promise<TaskRecord> {
  return invoke<TaskRecord>("create_task_from_security_finding", { request });
}
