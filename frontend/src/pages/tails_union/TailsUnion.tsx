import { Component, Show, createUniqueId } from "solid-js";
import { createStore, produce } from "solid-js/store";
import { CommandCard } from "~/components/common/CommandCard";
import { Button } from "~/components/ui/Button";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "~/components/ui/Card";
import NameSpace from "~/pages/index/components/NameSpace";
import { getAllTokens } from "~/lib/api";
import { rootToken, tokenRootNamespace, namespace } from "~/lib/state";
import { Copy, Check } from "lucide-solid";
import { executeTailsUnion, isLoading, isPolling, stopPolling } from "./lib";
import { onCleanup } from "solid-js";

interface NsItem {
  id: string;
  namespace: string[];
}

const TailsUnionPage: Component = () => {
  const [state, setState] = createStore({
    source: { id: createUniqueId(), namespace: [...namespace()] },
    target: { id: createUniqueId(), namespace: [...namespace()] },
    copied: false,
  });

  onCleanup(stopPolling);

  const updateSource = (ns: string[]) =>
    setState(
      "source",
      produce((s: NsItem) => {
        s.namespace = ns;
      })
    );
  const updateTarget = (ns: string[]) =>
    setState(
      "target",
      produce((t: NsItem) => {
        t.namespace = ns;
      })
    );

  const toNs = (ns: string[]) => ns.filter(Boolean).join("/");

  const buildPreview = () => {
    const src = toNs(state.source.namespace);
    const tgt = toNs(state.target.namespace);
    return `(transform
  (, (${src} ($h $t)))
  (, (${tgt} ($t)))
)`;
  };

  const copyPreview = () => {
    const expr = buildPreview();
    navigator.clipboard.writeText(expr);
    setState("copied", true);
    setTimeout(() => setState("copied", false), 2000);
  };

  const canRun = () =>
    state.source.namespace.filter(Boolean).length > 0 &&
    state.target.namespace.filter(Boolean).length > 0;

  const handleRun = async () => {
    await executeTailsUnion(state.source.namespace, state.target.namespace);
  };

  return (
    <div class="ml-10 mt-8">
      <CommandCard
        title="Tails Union"
        description="Emit tails from head-tail pairs into target namespace"
      >
        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <div class="lg:col-span-2 space-y-6">
            <Card class="border-l-4 border-l-primary">
              <CardHeader>
                <CardTitle>Source Namespace</CardTitle>
                <CardDescription>
                  Head-tail facts (e.g. (mammal (human (male 1))))
                </CardDescription>
              </CardHeader>
              <CardContent>
                <NameSpace
                  namespace={state.source.namespace}
                  setNamespace={updateSource}
                  rootToken={!!rootToken()}
                  tokenRootNamespace={tokenRootNamespace}
                  getAllTokens={getAllTokens}
                />
              </CardContent>
            </Card>

            <Card class="border-l-4 border-l-primary">
              <CardHeader>
                <CardTitle>Target Namespace</CardTitle>
                <CardDescription>
                  Where tail expressions will be written
                </CardDescription>
              </CardHeader>
              <CardContent>
                <NameSpace
                  namespace={state.target.namespace}
                  setNamespace={updateTarget}
                  rootToken={!!rootToken()}
                  tokenRootNamespace={tokenRootNamespace}
                  getAllTokens={getAllTokens}
                />
              </CardContent>
            </Card>

            <Button
              class="w-full"
              disabled={!canRun() || isLoading() || isPolling()}
              onClick={handleRun}
            >
              <Show when={isLoading() || isPolling()}>
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="24"
                  height="24"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  class="animate-spin mr-2 h-4 w-4"
                >
                  <path d="M21 12a9 9 0 1 1-6.219-8.56" />
                </svg>
              </Show>
              <Show
                when={isLoading()}
                fallback={
                  <Show when={isPolling()} fallback={"Run Tails Union"}>
                    Waiting for results...
                  </Show>
                }
              >
                Processing...
              </Show>
            </Button>
          </div>

          <div class="lg:col-span-1">
            <Card class="sticky top-4">
              <CardHeader>
                <CardTitle>S-Expression Preview</CardTitle>
                <CardDescription>
                  Drop head; keep tail ($h $t) → ($t)
                </CardDescription>
              </CardHeader>
              <CardContent>
                <pre class="text-sm font-mono bg-muted p-3 rounded overflow-auto">
                  {buildPreview()}
                </pre>
                <Button
                  variant="default"
                  size="sm"
                  onClick={copyPreview}
                  class="w-full mt-4"
                >
                  {state.copied ? (
                    <Check class="w-4 h-4 mr-2" />
                  ) : (
                    <Copy class="w-4 h-4 mr-2" />
                  )}
                  {state.copied ? "Copied!" : "Copy Expression"}
                </Button>
                <div class="mt-4 p-3 bg-muted/50 rounded text-sm text-muted-foreground">
                  Requires read on source and write on target. Backend executes
                  fixed transform pattern.
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      </CommandCard>
    </div>
  );
};

export default TailsUnionPage;
