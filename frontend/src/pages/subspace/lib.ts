import { createSignal } from "solid-js";
import { showToast } from "~/components/ui/Toast";
import { subspace, isPathClear } from "~/lib/api";
import { Mm2InputMultiWithNamespace, Item } from "~/lib/types";
import { refreshSpace } from "../load/lib";

export const [isLoading, setIsLoading] = createSignal(false);
export const [isPolling, setIsPolling] = createSignal(false);

let pollingIntervalId: NodeJS.Timeout | null = null;

export const stopPolling = () => {
  if (pollingIntervalId) clearInterval(pollingIntervalId);
  pollingIntervalId = null;
  setIsPolling(false);
};

export const startPolling = (spacePath: string) => {
  setIsPolling(true);
  pollingIntervalId = setInterval(async () => {
    try {
      const isClear = await isPathClear(spacePath);
      if (isClear) {
        stopPolling();
        showToast({
          title: "Subspace Operation Completed",
          description: "The space has been successfully processed.",
        });
        refreshSpace();
      }
    } catch {
      showToast({
        title: "Polling Error",
        description: "Failed to fetch operation status.",
        variant: "destructive",
      });
      stopPolling();
    }
  }, 3000);
};

export const formatNamespace = (namespace: string): string => {
  // Ensure namespace starts with a leading slash
  const formattedNamespace = namespace.startsWith("/")
    ? namespace
    : `/${namespace}`;
  console.log("Formatted namespace:", formattedNamespace);
  return formattedNamespace;
};

export const executeSubspace = async (
  patterns: Item[],
  templates: Item[],
  spacePath: string
) => {
  if (
    !patterns.some((p) => p.value.trim()) ||
    !templates.some((t) => t.value.trim())
  ) {
    showToast({
      title: "Error",
      description: "Please fill all required fields.",
      variant: "destructive",
    });
  }

  setIsLoading(true);
  stopPolling();

  try {
    if (!(await isPathClear(spacePath))) {
      showToast({
        title: "Space Busy",
        description: "The target space is currently busy. Please wait.",
        variant: "destructive",
      });
      setIsLoading(false);
      return false;
    }

    const input: Mm2InputMultiWithNamespace = {
      patterns: patterns.map((p) => ({
        kind: "pattern" as const,
        value: `(${p.value} $x)`,
        namespace: p.namespace.filter((n) => n !== "" && n !== "/"),
      })),
      templates: templates.map((t) => ({
        kind: "template" as const,
        value: t.value,
        namespace: t.namespace.filter((n) => n !== "" && n !== "/"),
      })),
    };

    const success = await subspace(input);
    if (success) {
      showToast({
        title: "Subspace Operation Initiated",
        description: "Waiting for results...",
      });
      startPolling(spacePath);
    } else {
      showToast({
        title: "Subspace Operation Failed",
        description: "Could not initiate the subspace operation.",
        variant: "destructive",
      });
      return false;
    }
  } catch (error) {
    const errorMessage =
      error instanceof Error ? error.message : "An unexpected error occurred.";
    showToast({
      title: "Error",
      description: errorMessage,
      variant: "destructive",
    });
    return false;
  } finally {
    setIsLoading(false);
  }
};
