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

type NsVal = { namespace: string[]; value: string };

export const executeRestriction = async (
  patterns: NsVal[], // expect exactly 2 entries
  templates: NsVal[] // expect at least 1; restriction uses exactly 1
) => {
  const current = (() => {
    const arr = namespace();
    const parts = arr.filter(Boolean);
    if (parts.length === 0) return "/";
    const p = parts.join("/");
    return p.endsWith("/") ? p : `${p}/`;
  })();

  // Validate counts
  if (patterns.length !== 2) {
    showToast({
      title: "Invalid Input",
      description: "Restriction needs exactly two patterns (paths and prefixes).",
      variant: "destructive",
    });
    return;
  }
  if (templates.length < 1) {
    showToast({
      title: "Invalid Input",
      description: "Provide one template for the output namespace.",
      variant: "destructive",
    });
    return;
  }

  // Normalize values only; keep namespace as arrays (backend expects sequence)
  const patternValues = patterns.map((p) => (p.value || "").trim());
  const firstTemplate = templates.find((t) => (t.value || "").trim().length > 0) || templates[0];
  const templateValue = (firstTemplate.value || "").trim();

  if (!patternValues.every((v) => v.length > 0) || templateValue.length === 0) {
    showToast({
      title: "Missing Values",
      description: "Fill both pattern expressions and the template expression.",
      variant: "destructive",
    });
    return;
  }
  const tgt = toPath(firstTemplate.namespace) || current;

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
      body: JSON.stringify({
        patterns: patterns.map((p, i) => ({ namespace: p.namespace, pattern: patternValues[i] })),
        templates: [{ namespace: firstTemplate.namespace, template: templateValue }],
      }),
    });

    if (ok) {
      showToast({ title: "Restriction Initiated", description: "Waiting for results..." });
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
    showToast({ title: "Error", description: errorMessage, variant: "destructive" });
  } finally {
    setIsLoading(false);
  }
};


