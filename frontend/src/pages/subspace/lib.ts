import { createSignal } from "solid-js";
import { showToast } from "~/components/ui/Toast";
import { isPathClear } from "~/lib/api";
import { refreshSpace } from "../load/lib";
import { rootToken } from "~/lib/state";
import { API_URL } from "~/lib/api";

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

export const executeSubspace = async (input: {
  source: string[];
  target: string[];
}) => {
  if (!input.source[0] || !input.source[1] || !input.target[0]) {
    showToast({
      title: "Error",
      description: "Please fill all required fields.",
      variant: "destructive",
    });
    return false;
  }

  setIsLoading(true);
  stopPolling();

  try {
    const spacePath = formatNamespace(input.target[0]);
    if (!(await isPathClear(spacePath))) {
      showToast({
        title: "Space Busy",
        description: "The target space is currently busy. Please wait.",
        variant: "destructive",
      });
      setIsLoading(false);
      return false;
    }

    const response = await fetch(`${API_URL}/spaces/subspace`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: rootToken() || "",
      },
      body: JSON.stringify(input),
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(errorText || "Subspace operation failed");
    }

    const result = await response.json();
    if (result === true) {
      showToast({
        title: "Subspace Operation Initiated",
        description: "Waiting for results...",
      });
      startPolling(spacePath);
      return true;
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
