import { For } from "solid-js";
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from "~/components/ui/Card";
import NameSpace from "~/pages/index/components/NameSpace";
import { TextField, TextFieldInput } from "~/components/ui/TextField";

interface Item { id: string; namespace: string[]; value: string; }

interface RestrictionInputProps {
  items: Item[]; // expect exactly 2: paths, prefixes
  updateItem: (id: string, field: "namespace" | "value", value: string | string[]) => void;
  accentColor: string;
  rootToken: boolean;
  tokenRootNamespace: () => string[];
  getAllTokens: () => Promise<{ namespace: string; description: string }[]>;
}

export function RestrictionInput(props: RestrictionInputProps) {
  return (
    <Card class={`border-l-4 border-l-${props.accentColor}`}>
      <CardHeader>
        <div class="flex items-center">
          <div class={`w-2 h-2 bg-${props.accentColor} rounded-full mr-2`} />
          <CardTitle>Sources</CardTitle>
        </div>
        <CardDescription>
          First row: path pattern (e.g. (path $a $b $v)). Second row: prefix pattern (e.g. (prefix $a $b)).
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div class="space-y-2">
          <For each={props.items}>
            {(item, i) => (
              <div class="flex gap-2 bg-neutral-800 p-3">
                <div class="flex-1 flex flex-col gap-2">
                  <div class="text-xs text-muted-foreground">
                    {i() === 0 ? "Path facts namespace" : "Prefix facts namespace"}
                  </div>
                  <NameSpace
                    namespace={item.namespace}
                    setNamespace={(ns) => props.updateItem(item.id, "namespace", ns)}
                    rootToken={props.rootToken}
                    tokenRootNamespace={props.tokenRootNamespace}
                    getAllTokens={props.getAllTokens}
                  />
                  <TextField>
                    <TextFieldInput
                      value={item.value}
                      onInput={(e) =>
                        props.updateItem(
                          item.id,
                          "value",
                          e.currentTarget.value
                        )
                      }
                      class="text-xs font-mono"
                      placeholder={
                        i() === 0 ? "(path $a $b $v)" : "(prefix $a $b)"
                      }
                    />
                  </TextField>
                </div>
              </div>
            )}
          </For>
        </div>
      </CardContent>
    </Card>
  );
}
