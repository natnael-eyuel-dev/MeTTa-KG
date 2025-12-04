import { Component, Show, onCleanup, createUniqueId } from "solid-js";
import { createStore, produce } from "solid-js/store";
import { CommandCard } from "~/components/common/CommandCard";
import { Button } from "~/components/ui/Button";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "~/components/ui/Card";
import { executeSubspace, isLoading, isPolling, stopPolling } from "./lib";
import { SubspaceInput as SubspaceInputComponent } from "./components/SubspaceInput";
import { Copy, Check } from "lucide-solid";
import { formatedNamespace, rootToken, tokenRootNamespace } from "~/lib/state";
import { getAllTokens } from "~/lib/api";

interface Item {
  id: string;
  namespace: string[];
  value: string;
}

const SubspacePage: Component = () => {
  const [state, setState] = createStore({
    patterns: [{ id: createUniqueId(), namespace: [""], value: "" }],
    templates: [{ id: createUniqueId(), namespace: [""], value: "$x" }],
    copied: false,
  });

  onCleanup(stopPolling);

  const buildNestedPath = (namespace: string[], inner: string): string => {
    if (namespace.length === 0) return inner;

    const [first, ...rest] = namespace;
    if (rest.length === 0) {
      return `(${first} ${inner})`;
    }

    return `(${first} ${buildNestedPath(rest, inner)})`;
  };

  const buildSubspaceSExpr = (
    sourcePatterns: Item[],
    targetTemplates: Item[]
  ) => {
    const filterSlash = (ns: string[]): string[] => {
      if (ns.length > 1) {
        if (ns[0] === "/" || ns[0] === "") {
          return ns.slice(1);
        }
      }
      return ns;
    };
    const sourcePattern = sourcePatterns[0] || {
      namespace: [""],
      value: "",
    };
    const targetTemplate = targetTemplates[0] || {
      namespace: [""],
      value: "",
    };

    const sourcePath =
      sourcePattern.namespace.length > 0
        ? buildNestedPath(
            filterSlash(sourcePattern.namespace),
            `(${sourcePattern.value} $x)`
          )
        : `(${sourcePattern.value} $x)`;

    const targetPath =
      targetTemplate.namespace.length > 0
        ? buildNestedPath(filterSlash(targetTemplate.namespace), "$x")
        : "$x";

    return `(transform
    (, ${sourcePath})
    (, ${targetPath})
)`;
  };

  const handleSubspace = async () => {
    executeSubspace(state.patterns, state.templates, formatedNamespace());
  };

  const addSourcePattern = () => {
    setState("patterns", (prev) => [
      ...prev,
      { id: createUniqueId(), namespace: [""], value: "" },
    ]);
  };

  const removeSourcePattern = (id: string) => {
    setState("patterns", (prev) => prev.filter((p) => p.id !== id));
  };

  const updateSourcePattern = (
    id: string,
    field: "namespace" | "value",
    value: string | string[]
  ) => {
    setState(
      "patterns",
      produce((patterns) => {
        const item = patterns.find((p) => p.id === id);
        if (item) {
          if (field === "namespace") {
            item.namespace = value as string[];
          } else {
            item.value = value as string;
          }
        }
      })
    );
  };

  const addTargetTemplate = () => {
    setState("templates", (prev) => [
      ...prev,
      { id: createUniqueId(), namespace: [""], value: "" },
    ]);
  };

  const removeTargetTemplate = (id: string) => {
    setState("templates", (prev) => prev.filter((t) => t.id !== id));
  };

  const updateTargetTemplate = (
    id: string,
    field: "namespace" | "value",
    value: string | string[]
  ) => {
    setState(
      "templates",
      produce((templates) => {
        const item = templates.find((t) => t.id === id);
        if (item) {
          if (field === "namespace") {
            item.namespace = value as string[];
          } else {
            item.value = value as string;
          }
        }
      })
    );
  };

  const canExecute = () => {
    const sourcePattern = state.patterns[0];
    const targetTemplate = state.templates[0];

    return (
      sourcePattern &&
      targetTemplate &&
      sourcePattern.namespace.length > 0 &&
      sourcePattern.value.trim() &&
      targetTemplate.namespace.length > 0
    );
  };

  const copyExpression = () => {
    const sExpr = buildSubspaceSExpr(state.patterns, state.templates);
    navigator.clipboard.writeText(sExpr);
    setState("copied", true);
    setTimeout(() => setState("copied", false), 2000);
  };

  return (
    <div class="ml-10 mt-8">
      <CommandCard
        title="Subspace Operation"
        description="Extract all tails under a given prefix from a source namespace into a target namespace."
      >
        <div class="space-y-6">
          <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
            {/* Builder - 2/3 */}
            <div class="lg:col-span-2 space-y-6">
              <SubspaceInputComponent
                type="patterns"
                items={state.patterns}
                addItem={addSourcePattern}
                removeItem={removeSourcePattern}
                updateItem={updateSourcePattern}
                accentColor="green-500"
                rootToken={rootToken()}
                tokenRootNamespace={tokenRootNamespace}
                getAllTokens={getAllTokens}
              />
              <SubspaceInputComponent
                type="templates"
                items={state.templates}
                addItem={addTargetTemplate}
                removeItem={removeTargetTemplate}
                updateItem={updateTargetTemplate}
                accentColor="green-500"
                rootToken={rootToken()}
                tokenRootNamespace={tokenRootNamespace}
                getAllTokens={getAllTokens}
              />
            </div>
            {/* Preview - 1/3 */}
            <div class="lg:col-span-1">
              <Card class="sticky top-4">
                <CardHeader>
                  <CardTitle>S-Expression Preview</CardTitle>
                  <CardDescription>
                    Live output of your subspace operation
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <pre class="text-sm font-mono bg-muted p-3 rounded overflow-auto">
                    {buildSubspaceSExpr(state.patterns, state.templates)}
                  </pre>
                  <Button
                    variant="default"
                    size="sm"
                    onClick={copyExpression}
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
                    Subspace extracts tails under a given prefix.
                  </div>
                </CardContent>
              </Card>
            </div>
          </div>
        </div>

        <Button
          onClick={handleSubspace}
          disabled={isLoading() || isPolling() || !canExecute()}
          class="inline-flex items-center justify-center w-[180px] h-10 mt-4"
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
              <Show when={isPolling()} fallback={"Run Subspace"}>
                Waiting for results...
              </Show>
            }
          >
            Processing subspace...
          </Show>
        </Button>
      </CommandCard>
    </div>
  );
};

export default SubspacePage;
