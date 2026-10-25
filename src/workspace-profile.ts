export const WORKSPACE_PROFILES = [
  {
    id: "developer",
    label: "Developer",
    description: "Code, repositories, projects and tasks.",
    pinned: ["Projects", "Tasks", "Knowledge"],
    layout: "build",
  },
  {
    id: "network_noc",
    label: "Network / NOC",
    description: "Network visibility and operational follow-up.",
    pinned: ["Operations", "Tasks", "Knowledge"],
    layout: "monitor",
  },
  {
    id: "sysadmin",
    label: "Sysadmin",
    description: "System administration and maintenance planning.",
    pinned: ["Toolbox", "Operations", "Tasks"],
    layout: "operations",
  },
  {
    id: "security_research",
    label: "Security / Research",
    description: "Authorized security work, evidence and remediation planning.",
    pinned: ["Security", "Projects", "Knowledge"],
    layout: "research",
  },
  {
    id: "intelligence_research",
    label: "Intelligence / Research",
    description: "Research notes, source review and task follow-up.",
    pinned: ["Intelligence", "Knowledge", "Tasks"],
    layout: "research",
  },
  {
    id: "custom",
    label: "Custom",
    description: "A neutral layout you can tailor later.",
    pinned: ["Home", "Tasks", "Knowledge"],
    layout: "custom",
  },
] as const;

export type WorkspaceProfileId = (typeof WORKSPACE_PROFILES)[number]["id"];
export type WorkspaceProfile = (typeof WORKSPACE_PROFILES)[number];

const PROFILE_STORAGE_KEY = "moonships.workspace-profile.v1";

export function getWorkspaceProfile(id: string | null | undefined): WorkspaceProfile {
  return WORKSPACE_PROFILES.find((profile) => profile.id === id) ?? WORKSPACE_PROFILES[0];
}

export function readWorkspaceProfile(
  storage?: Pick<Storage, "getItem">,
): WorkspaceProfileId | null {
  try {
    const value = storage?.getItem(PROFILE_STORAGE_KEY);
    return WORKSPACE_PROFILES.some((profile) => profile.id === value)
      ? (value as WorkspaceProfileId)
      : null;
  } catch {
    return null;
  }
}

export function writeWorkspaceProfile(
  profile: WorkspaceProfileId,
  storage?: Pick<Storage, "setItem">,
): boolean {
  try {
    storage?.setItem(PROFILE_STORAGE_KEY, profile);
    return Boolean(storage);
  } catch {
    return false;
  }
}
