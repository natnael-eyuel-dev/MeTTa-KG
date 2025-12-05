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
import { getAllTokens } from "~/lib/api";
import { rootToken, tokenRootNamespace } from "~/lib/state";
import { Copy, Check } from "lucide-solid";
import { executeTailsUnion, isLoading, isPolling, stopPolling } from "./lib";
import { onCleanup } from "solid-js";
import { TailsUnionInput as TailsUnionInputComponent } from "./components/TailsUnionInput";

const TailsUnionPage: Component = () => {
  const [state, setState] = createStore({
    patterns: { id: createUniqueId(), namespace: [""] },
    templates: { id: createUniqueId(), namespace: [""] },
    copied: false,
  });

  onCleanup(stopPolling);

  const convertToSExpression = (
    path: string[],
    variable: string = "$x"
  ): string => {
    if (path.length === 0) {
      return `${variable}`;
    }

    if (path.length >= 1) {
      if (path[0] === "" || path[0] === "/") {
        if (path.length === 1) {
          return `${variable}`;
        }
        path = path.slice(1);
      }
    }

    let result = "";

    // Build the nested structure from the end to the beginning
    for (let i = path.length - 1; i >= 0; i--) {
      if (i === path.length - 1) {
        // Last element contains the variable
        result = `(${path[i]} ${variable})`;
      } else {
        // Wrap previous result in parentheses
        result = `(${path[i]} ${result})`;
      }
    }

    return result;
  };
  const buildPreview = () => {
    const patternExprs: string = convertToSExpression(
      state.patterns.namespace,
      "($x $y)"
    );
    const templatesExprs: string = convertToSExpression(
      state.templates.namespace,
      "$y"
    );

    return `(transform\n (, ${patternExprs})\n (, ${templatesExprs})\n)`;
  };

  const copyPreview = () => {
    const expr = buildPreview();
    navigator.clipboard.writeText(expr);
    setState("copied", true);
    setTimeout(() => setState("copied", false), 2000);
  };

  const handleRun = async () => {
    await executeTailsUnion(
      state.patterns.namespace,
      state.templates.namespace
    );
  };

  const addPattern = () => {
    setState("patterns", { id: createUniqueId(), namespace: [""] });
  };

  const updatePattern = (ids: string, field: "namespace", value: string[]) => {
    setState(
      "patterns",
      produce((patterns) => {
        if (patterns.id === ids) {
          patterns[field] = value;
        }
      })
    );
  };

  const addTemplate = () => {
    setState("templates", { id: createUniqueId(), namespace: [""] });
  };

  const updateTemplate = (ids: string, field: "namespace", value: string[]) => {
    setState(
      "templates",
      produce((templates) => {
        if (templates.id === ids) {
          templates[field] = value;
        }
      })
    );
  };

  return (
    <div class="ml-10 mt-8">
      <CommandCard
        title="Tails Union"
        description="Emit tails from head-tail pairs into templates namespace"
      >
        <div class="space-y-6">
          {/* Responsive Layout */}
          <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
            {/* Builder - 2/3 */}
            <div class="lg:col-span-2 space-y-6">
              <TailsUnionInputComponent
                type="patterns"
                items={state.patterns}
                addItem={addPattern}
                updateItem={updatePattern}
                accentColor="primary"
                rootToken={rootToken()}
                tokenRootNamespace={tokenRootNamespace}
                getAllTokens={getAllTokens}
              />
              <TailsUnionInputComponent
                type="templates"
                items={state.templates}
                addItem={addTemplate}
                updateItem={updateTemplate}
                accentColor="primary"
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
                </CardContent>
              </Card>
            </div>
          </div>
        </div>

        <Button
          onClick={handleRun}
          disabled={isLoading() || isPolling()}
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
              <Show when={isPolling()} fallback={"Run Tails Union"}>
                Waiting for results...
              </Show>
            }
          >
            Performing Tails Union...
          </Show>
        </Button>
      </CommandCard>
    </div>
  );
};

export default TailsUnionPage;
