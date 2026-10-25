export type ModuleAvailability =
  | "available"
  | "disabled"
  | "unsupported"
  | "not_configured"
  | "degraded";

export type ModuleAvailabilityView = {
  label: string;
  description: string;
  isOperational: boolean;
};

const availabilityViews: Record<ModuleAvailability, ModuleAvailabilityView> = {
  available: {
    label: "Available",
    description: "The module is configured and ready to use.",
    isOperational: true,
  },
  disabled: {
    label: "Disabled",
    description: "The module is intentionally disabled in settings.",
    isOperational: false,
  },
  unsupported: {
    label: "Unsupported",
    description: "This module is not supported on the current platform.",
    isOperational: false,
  },
  not_configured: {
    label: "Not configured",
    description: "Required setup is missing; no operation has been attempted.",
    isOperational: false,
  },
  degraded: {
    label: "Degraded",
    description: "The module is configured but has a current operational issue.",
    isOperational: false,
  },
};

export function describeModuleAvailability(
  availability: ModuleAvailability,
): ModuleAvailabilityView {
  return availabilityViews[availability];
}
