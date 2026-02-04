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
import { RestrictionInput } from "./components/RestrictionInput";
import { getAllTokens } from "~/lib/api";
import { rootToken, tokenRootNamespace } from "~/lib/state";
import { Copy, Check } from "lucide-solid";
import { isLoading, isPolling, executeRestriction, stopPolling } from "./lib";

interface Item {
  id: string;
  namespace: string[];
}

const RestrictionPage: Component = () => {
  const [state, setState] = createStore({
    patterns: [
      { id: createUniqueId(), namespace: [""] },
      { id: createUniqueId(), namespace: [""] },
    ],
    templates: [{ id: createUniqueId(), namespace: [""] }],
    copied: false,
  });

  onCleanup(stopPolling);

  const updatePattern = (id: string, value: string[]) => {
    setState(
      "patterns",
      produce((patterns: Item[]) => {
        const item = patterns.find((p) => p.id === id);
        if (item) {
          item.namespace = value;
        }
      })
    );
  };

  const updateTemplate = (id: string, value: string[]) => {
    setState(
      "templates",
      produce((templates: Item[]) => {
        const item = templates.find((t) => t.id === id);
        if (item) {
          item.namespace = value;
        }
      })
    );
  };

  const canSubmit = () => {
    // Restriction requires exactly 2 sources (paths, prefixes) and 1 target.
    const hasPatternValue = state.patterns.length === 2;
    const hasTemplateValue = state.templates.length === 1;
    return hasPatternValue && hasTemplateValue;
  };

  const buildTransformPreview = (patterns: Item[], templates: Item[]) => {
    const wrapNs = (ns: string[], placeholder: string, inner: string) => {
      const parts = ns.filter(Boolean);
      if (parts.length === 0) {
        return `(${placeholder} ${inner})`;
      }
      return parts.reduceRight((acc, part) => `(${part} ${acc})`, inner);
    };

    const pathsExpr = wrapNs(
      patterns[0]?.namespace || [],
      "<paths>",
      "(path $a $b $v)"
    );
    const prefixesExpr = wrapNs(
      patterns[1]?.namespace || [],
      "<prefixes>",
      "(prefix $a $b)"
    );
    const outExpr = wrapNs(
      templates[0]?.namespace || [],
      "<out>",
      "(path $a $b $v)"
    );

    return `(transform
    (, ${pathsExpr} ${prefixesExpr})
    (, ${outExpr})
)`;
  };

  const copyExpression = () => {
    const expr = buildTransformPreview(state.patterns, state.templates);
    navigator.clipboard.writeText(expr);
    setState("copied", true);
    setTimeout(() => setState("copied", false), 2000);
  };

  const handleRestriction = async () => {
    await executeRestriction(
      state.patterns.map((p: Item) => p.namespace),
      state.templates[0]?.namespace || [""]
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
              type="patterns"
              items={state.patterns as Item[]}
              updateItem={updatePattern}
              accentColor="primary"
              rootToken={!!rootToken()}
              tokenRootNamespace={tokenRootNamespace}
              getAllTokens={getAllTokens}
              description="Select the paths space (X) and prefixes space (Y)"
            />

            <RestrictionInput
              type="templates"
              items={state.templates as Item[]}
              updateItem={updateTemplate}
              accentColor="primary"
              rootToken={!!rootToken()}
              tokenRootNamespace={tokenRootNamespace}
              getAllTokens={getAllTokens}
              description="Select the output target namespace"
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
                  {buildTransformPreview(state.patterns, state.templates)}
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
              </CardContent>
            </Card>
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
        </div>
      </CommandCard>
    </div>
  );
};

export default RestrictionPage;
