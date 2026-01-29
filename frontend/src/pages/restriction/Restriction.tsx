import { Component, Show, onCleanup, createUniqueId } from "solid-js";
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
import { TransformInput as TransformInputComponent } from "~/pages/transform/components/TransformInput";
import { RestrictionInput } from "./components/RestrictionInput";
import { getAllTokens } from "~/lib/api";
import { rootToken, tokenRootNamespace, namespace } from "~/lib/state";
import { Copy, Check } from "lucide-solid";
import { isLoading, isPolling, executeRestriction, stopPolling } from "./lib";

interface Item {
  id: string;
  namespace: string[];
  value: string;
}

const RestrictionPage: Component = () => {
  const [state, setState] = createStore({
    patterns: [
      { id: createUniqueId(), namespace: [...namespace()], value: "" },
      { id: createUniqueId(), namespace: [...namespace()], value: "" },
    ],
    templates: [
      { id: createUniqueId(), namespace: [...namespace()], value: "" },
    ],
    copied: false,
  });

  onCleanup(stopPolling);

  const updatePattern = (
    id: string,
    field: "namespace" | "value",
    value: string | string[]
  ) => {
    setState(
      "patterns",
      produce((patterns: Item[]) => {
        const item = patterns.find((p) => p.id === id);
        if (item) {
          if (field === "namespace") item.namespace = value as string[];
          else item.value = value as string;
        }
      })
    );
  };

  const addTemplate = () => {
    setState("templates", (prev) => [
      ...prev,
      { id: createUniqueId(), namespace: [""], value: "" },
    ]);
  };
  const removeTemplate = (id: string) => {
    setState("templates", (prev) => prev.filter((t) => t.id !== id));
  };
  const updateTemplate = (
    id: string,
    field: "namespace" | "value",
    value: string | string[]
  ) => {
    setState(
      "templates",
      produce((templates: Item[]) => {
        const item = templates.find((t) => t.id === id);
        if (item) {
          if (field === "namespace") item.namespace = value as string[];
          else item.value = value as string;
        }
      })
    );
  };

  const canSubmit = () => {
    // Need both canonical patterns and a template output expression
    const patternsOk = state.patterns.every(
      (p: Item) => (p.value || "").trim().length > 0
    );
    const templateOk = state.templates.some(
      (t: Item) => (t.value || "").trim().length > 0
    );
    return patternsOk && templateOk;
  };

  const buildTransformPreview = () => {
    const wrap = (v: string) => {
      const trimmed = v.trim();
      if (!trimmed) return "()";
      return trimmed.startsWith("(") ? trimmed : `(${trimmed})`;
    };
    const patternParts = state.patterns.map((p: Item) => wrap(p.value || ""));
    const templateParts = state.templates.map((t: Item) => wrap(t.value || ""));
    const patternsExpr = `(,  ${patternParts.join(" ")} )`;
    const templatesExpr = `(, ${templateParts.join(" ")} )`;
    return `(transform\n    ${patternsExpr}\n    ${templatesExpr}\n)`;
  };

  const copyExpression = () => {
    const expr = buildTransformPreview();
    navigator.clipboard.writeText(expr);
    setState("copied", true);
    setTimeout(() => setState("copied", false), 2000);
  };

  const handleRestriction = async () => {
    await executeRestriction(
      state.patterns as unknown as { namespace: string[]; value: string }[],
      state.templates as unknown as { namespace: string[]; value: string }[]
    );
  };

  return (
    <div class="ml-10 mt-8">
      <CommandCard
        title="Restriction"
        description="Filter path entries by prefix set into target namespace"
      >
        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <div class="lg:col-span-2 space-y-6">
            <RestrictionInput
              items={state.patterns as Item[]}
              updateItem={updatePattern}
              accentColor="primary"
              rootToken={!!rootToken()}
              tokenRootNamespace={tokenRootNamespace}
              getAllTokens={getAllTokens}
            />

            <TransformInputComponent
              type="templates"
              items={state.templates as Item[]}
              addItem={addTemplate}
              removeItem={removeTemplate}
              updateItem={updateTemplate}
              accentColor="primary"
              rootToken={!!rootToken()}
              tokenRootNamespace={tokenRootNamespace}
              getAllTokens={getAllTokens}
              description="Template for surviving path entries"
            />
          </div>

          <div class="lg:col-span-1">
            <Card class="sticky top-4">
              <CardHeader>
                <CardTitle>S-Expression Preview</CardTitle>
                <CardDescription>
                  Conceptual transform for restriction
                </CardDescription>
              </CardHeader>
              <CardContent>
                <pre class="text-sm font-mono bg-muted p-3 rounded overflow-auto">
                  {buildTransformPreview()}
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
                  Restriction runs server-side using selected namespaces.
                </div>
                <Button
                  class="w-full mt-4"
                  disabled={!canSubmit() || isLoading() || isPolling()}
                  onClick={handleRestriction}
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
                      <Show when={isPolling()} fallback={"Run Restriction"}>
                        Waiting for results...
                      </Show>
                    }
                  >
                    Processing...
                  </Show>
                </Button>
              </CardContent>
            </Card>
          </div>
        </div>
      </CommandCard>
    </div>
  );
};

export default RestrictionPage;
