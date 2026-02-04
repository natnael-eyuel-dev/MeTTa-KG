import { createSignal } from "solid-js";
import { isPathClear, request } from "~/lib/api";
import { namespace } from "~/lib/state";
import { showToast } from "~/components/ui/Toast";
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
          title: "Restriction Completed",
          description: "Results written to the target namespace.",
        });
        refreshSpace();
      }
    } catch {
      showToast({
        title: "Polling Error",
        description: "Failed to fetch transformation status.",
        variant: "destructive",
      });
      stopPolling();
    }
  }, 3000);
};

const toPath = (ns: string[]) => {
  const parts = ns.filter(Boolean);
  if (parts.length === 0) return "";
  const p = parts.join("/");
  return p.endsWith("/") ? p : `${p}/`;
};

export const executeRestriction = async (
  sources: string[][], // expect exactly 2 entries: [pathsNs, prefixesNs]
  target: string[] // output namespace
) => {
  const current = (() => {
    const arr = namespace();
    const parts = arr.filter(Boolean);
    if (parts.length === 0) return "/";
    const p = parts.join("/");
    return p.endsWith("/") ? p : `${p}/`;
  })();

  // Validate counts
  if (sources.length !== 2) {
    showToast({
      title: "Invalid Input",
      description:
        "Restriction needs exactly two source namespaces (paths and prefixes).",
      variant: "destructive",
    });
    return;
  }
  const src = sources
    .map(toPath)
    .map((s) => s || current)
    .filter(Boolean);
  const tgt = toPath(target) || current;

  if (src.length !== 2 || !tgt) {
    showToast({
      title: "Invalid Input",
      description: "Provide one template for the output namespace.",
      variant: "destructive",
    });
    return;
  }

  setIsLoading(true);
  stopPolling();

  try {
    if (!(await isPathClear(tgt))) {
      showToast({
        title: "Space Busy",
        description: "The target space is currently busy. Please wait.",
        variant: "destructive",
      });
      setIsLoading(false);
      return;
    }

    const ok = await request<boolean>("/spaces/restriction", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ source: src, target: [tgt] }),
    });

    if (ok) {
      showToast({
        title: "Restriction Initiated",
        description: "Waiting for results...",
      });
      startPolling(tgt);
    } else {
      showToast({
        title: "Restriction Failed",
        description: "Could not initiate the restriction.",
        variant: "destructive",
      });
    }
  } catch (error) {
    const errorMessage =
      error instanceof Error ? error.message : "An unexpected error occurred.";
    showToast({
      title: "Error",
      description: errorMessage,
      variant: "destructive",
    });
  } finally {
    setIsLoading(false);
  }
};
