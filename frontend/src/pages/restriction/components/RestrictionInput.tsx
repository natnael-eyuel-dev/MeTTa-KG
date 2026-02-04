import { For, Show } from "solid-js";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "~/components/ui/Card";
import NameSpace from "~/pages/index/components/NameSpace";

type Token = {
  namespace: string;
  description: string;
};

export interface Item {
  id: string;
  namespace: string[];
}

interface RestrictionInputProps {
  type: "patterns" | "templates";
  items: Item[]; // patterns: 2 items (paths, prefixes); templates: 1 item (target)
  updateItem: (id: string, ns: string[]) => void;
  accentColor: string;
  rootToken: boolean;
  tokenRootNamespace: () => string[];
  getAllTokens: () => Promise<Token[]>;
  description: string;
}

export function RestrictionInput(props: RestrictionInputProps) {
  const title = props.type === "patterns" ? "Patterns" : "Templates";

  return (
    <Card class={`border-l-4 border-l-${props.accentColor}`}>
      <CardHeader>
        <div class="flex items-center">
          <div class={`w-2 h-2 bg-${props.accentColor} rounded-full mr-2`} />
          <CardTitle>{title}</CardTitle>
        </div>
        <CardDescription>{props.description}</CardDescription>
      </CardHeader>
      <CardContent>
        <div class="space-y-2">
          <For each={props.items}>
            {(item, i) => (
              <div class="flex gap-2 bg-neutral-800 p-3">
                <div class="flex-1 flex flex-col gap-2">
                  <Show when={props.type === "patterns"}>
                    <div class="text-xs text-muted-foreground">
                      {i() === 0
                        ? "Paths (X): (path ...)"
                        : "Prefixes (Y): (prefix ...)"}
                    </div>
                  </Show>
                  <Show when={props.type === "templates"}>
                    <div class="text-xs text-muted-foreground">
                      Target (Out): surviving (path ...) facts
                    </div>
                  </Show>
                  <NameSpace
                    namespace={item.namespace}
                    setNamespace={(ns) => props.updateItem(item.id, ns)}
                    rootToken={props.rootToken}
                    tokenRootNamespace={props.tokenRootNamespace}
                    getAllTokens={props.getAllTokens}
                  />
                </div>
              </div>
            )}
          </For>
        </div>
      </CardContent>
    </Card>
  );
}
