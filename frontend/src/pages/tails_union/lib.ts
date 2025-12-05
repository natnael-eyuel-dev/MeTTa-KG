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

const currentRoot = () => {
  const arr = namespace();
  const parts = arr.filter(Boolean);
  if (parts.length === 0) return "/";
  const p = parts.join("/");
  return p.endsWith("/") ? p : `${p}/`;
};

const toPath = (ns: string[]) => {
  const parts = ns.filter(Boolean);
  if (parts.length === 0) return currentRoot();
  const p = parts.join("/");
  return p.endsWith("/") ? p : `${p}/`;
};

export const startPolling = (spacePath: string) => {
  setIsPolling(true);
  pollingIntervalId = setInterval(async () => {
    try {
      const clear = await isPathClear(spacePath);
      if (clear) {
        stopPolling();
        showToast({
          title: "TailsUnion Completed",
          description: "Tail expressions written to target namespace.",
        });
        refreshSpace();
      }
    } catch (error) {
      const errorMessage =
        error instanceof Error
          ? error.message
          : "An unexpected error occurred.";
      showToast({
        title: "Polling Error",
        description: errorMessage,
        variant: "destructive",
      });
      stopPolling();
    }
  }, 3000);
};

export const executeTailsUnion = async (
  sourceNs: string[],
  targetNs: string[]
) => {
  const sourcePath = toPath(sourceNs).replace(/\/$/, "");
  const targetPath = toPath(targetNs).replace(/\/$/, "");

  if (!sourcePath || !targetPath) {
    showToast({
      title: "Invalid Namespaces",
      description: "Select both source and target namespaces.",
      variant: "destructive",
    });
    return;
  }

  setIsLoading(true);
  stopPolling();

  try {
    if (!(await isPathClear(targetPath + "/"))) {
      showToast({
        title: "Space Busy",
        description: "Target namespace busy; wait for completion.",
        variant: "destructive",
      });
      setIsLoading(false);
      return;
    }

    const ok = await request<boolean>("/spaces/tails_union", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ source: [sourcePath], target: [targetPath] }),
    });

    if (ok) {
      showToast({
        title: "TailsUnion Started",
        description: "Polling for completion...",
      });
      startPolling(targetPath + "/");
    } else {
      showToast({
        title: "TailsUnion Failed",
        description: "Could not start tails union transform.",
        variant: "destructive",
      });
    }
  } catch (error) {
    const msg = error instanceof Error ? error.message : "Unexpected error";
    showToast({ title: "Error", description: msg, variant: "destructive" });
  } finally {
    setIsLoading(false);
  }
};
