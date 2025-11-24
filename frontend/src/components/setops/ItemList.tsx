import { For, Show, createEffect, createSignal } from "solid-js";
import { Button } from "~/components/ui/Button";
import { TextField, TextFieldInput } from "~/components/ui/TextField";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "~/components/ui/DropdownMenu";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "~/components/ui/Card";
import ChevronDown from "lucide-solid/icons/chevron-down";
import { Plus, Trash2 } from "lucide-solid";

export interface Item {
  id: string;
  namespace: string[];
  value?: string;
}

export interface ItemListProps {
  title: string;
  description?: string;
  items: Item[];
  onAdd?: () => void;
  onRemove?: (id: string) => void;
  onUpdateNamespace: (id: string, ns: string[]) => void;
  onUpdateValue?: (id: string, value: string) => void;
  minItems?: number;
  showValue?: boolean;
  itemLabels?: (index: number) => string;
  rootToken: boolean;
  tokenRootNamespace: () => string[];
  getAllTokens: () => Promise<{ namespace: string; description: string }[]>;
  addButtonLabel?: string;
  disableAdd?: boolean;
}

export function ItemList(props: ItemListProps) {
  const canRemove = (length: number) =>
    props.minItems ? length > props.minItems : length > 1;

  const [tokens, setTokens] = createSignal<
    { namespace: string; description: string }[]
  >([]);
  const [filter, setFilter] = createSignal("");
  const [loaded, setLoaded] = createSignal(false);

  createEffect(() => {
    if (!loaded() && props.rootToken) {
      props
        .getAllTokens()
        .then((all) => {
          const unique = new Map<string, string>();
          for (const t of all) {
            const ns =
              t.namespace.endsWith("/") && t.namespace.length > 1
                ? t.namespace.slice(0, -1)
                : t.namespace;
            if (!unique.has(ns)) unique.set(ns, t.description);
          }
          setTokens(
            Array.from(unique.entries()).map(([namespace, description]) => ({
              namespace,
              description,
            }))
          );
          setLoaded(true);
        })
        .catch(() => setLoaded(true));
    }
  });

  const filtered = () => {
    const f = filter().trim().toLowerCase();
    if (!f) return tokens();
    return tokens().filter((t) => t.namespace.toLowerCase().includes(f));
  };

  const currentDisplay = (ns: string[]) => {
    const parts = ns.slice(1);
    if (parts.length === 0) return "/";
    return "/" + parts.join("/") + "/";
  };

  const selectNamespace = (id: string, ns: string) => {
    const parts = ns.split("/").filter((p) => p.length > 0);
    props.onUpdateNamespace(id, ["", ...parts]);
  };

  return (
    <Card class="border-l-4 border-l-primary">
      <CardHeader>
        <div class="flex items-center">
          <div class="w-2 h-2 bg-primary rounded-full mr-2" />
          <CardTitle>{props.title}</CardTitle>
        </div>
        <Show when={props.description}>
          <CardDescription>{props.description}</CardDescription>
        </Show>
      </CardHeader>
      <CardContent>
        <div class="space-y-2">
          <For each={props.items}>
            {(item, i) => (
              <div class="flex gap-2 bg-neutral-800 p-3">
                <div class="flex-1 flex flex-col gap-2">
                  <Show when={props.itemLabels}>
                    <div class="text-xs text-muted-foreground">
                      {props.itemLabels!(i())}
                    </div>
                  </Show>
                  <TextField>
                    <DropdownMenu>
                      <DropdownMenuTrigger
                        as="button"
                        type="button"
                        class="w-full flex items-center justify-between px-3 py-2 border rounded-md bg-background hover:bg-accent"
                        aria-label="Select namespace"
                      >
                        <span
                          class="text-sm truncate"
                          title={currentDisplay(item.namespace)}
                        >
                          {currentDisplay(item.namespace)}
                        </span>
                        <ChevronDown class="h-4 w-4 opacity-50" />
                      </DropdownMenuTrigger>
                      <DropdownMenuContent class="w-full max-h-64 overflow-y-auto">
                        <div class="px-2 py-1">
                          <TextFieldInput
                            placeholder="Filter namespaces..."
                            value={filter()}
                            aria-label="Filter namespaces"
                            onInput={(e) => setFilter(e.currentTarget.value)}
                            class="h-8 text-xs"
                          />
                        </div>
                        <Show
                          when={filtered().length > 0}
                          fallback={
                            <div class="px-3 py-2 text-xs text-muted-foreground">
                              No matches.
                            </div>
                          }
                        >
                          <For each={filtered()}>
                            {(nsItem) => (
                              <DropdownMenuItem
                                onSelect={() =>
                                  selectNamespace(item.id, nsItem.namespace)
                                }
                                class="flex flex-col items-start"
                              >
                                <span class="font-mono text-xs">
                                  {nsItem.namespace.endsWith("/")
                                    ? nsItem.namespace
                                    : nsItem.namespace + "/"}
                                </span>
                                <Show when={nsItem.description}>
                                  <span class="text-[10px] text-muted-foreground max-w-full truncate">
                                    {nsItem.description}
                                  </span>
                                </Show>
                              </DropdownMenuItem>
                            )}
                          </For>
                        </Show>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </TextField>
                  <Show when={props.showValue}>
                    <TextField class="flex-1">
                      <TextFieldInput
                        value={item.value || ""}
                        onInput={(e) =>
                          props.onUpdateValue?.(item.id, e.currentTarget.value)
                        }
                        placeholder="Value expression"
                        class="text-sm font-mono resize-none"
                      />
                    </TextField>
                  </Show>
                </div>
                <Show when={props.onRemove}>
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    onClick={() => props.onRemove!(item.id)}
                    disabled={!canRemove(props.items.length)}
                    class="text-destructive hover:text-destructive self-center"
                    aria-label="Remove item"
                  >
                    <Trash2 class="w-4 h-4" />
                  </Button>
                </Show>
              </div>
            )}
          </For>
          <Show when={props.onAdd && !props.disableAdd}>
            <hr class="my-4" />
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => props.onAdd?.()}
              class="w-full"
              aria-label="Add item"
            >
              <Plus class="w-4 h-4 mr-2" />
              {props.addButtonLabel || `Add ${props.title}`}
            </Button>
          </Show>
        </div>
      </CardContent>
    </Card>
  );
}
