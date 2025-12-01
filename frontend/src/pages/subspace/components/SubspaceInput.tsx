import { For } from "solid-js";
import { TextField, TextFieldInput } from "~/components/ui/TextField";
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

interface Item {
  id: string;
  namespace: string[];
  value: string;
}

interface SubspaceInputProps {
  type: "source" | "target";
  items: Item[];
  addItem: () => void;
  removeItem: (id: string) => void;
  updateItem: (
    id: string,
    field: "namespace" | "value",
    value: string | string[]
  ) => void;
  accentColor: string;
  rootToken: boolean;
  tokenRootNamespace: () => string[];
  getAllTokens: () => Promise<Token[]>;
}

export function SubspaceInput(props: SubspaceInputProps) {
  const title = props.type === "source" ? "Pattern" : "Template";
  const description =
    props.type === "source"
      ? "Define the source namespace and prefix pattern"
      : "Define the target namespace for the subspace";

  const showValueField = props.type === "source";

  return (
    <Card class="border-l-4" style="border-left-color: rgb(34 197 94);">
      <CardHeader>
        <div class="flex items-center">
          <div
            class="w-2 h-2 rounded-full mr-2"
            style="background-color: rgb(34 197 94);"
          ></div>
          <CardTitle>{title}</CardTitle>
        </div>
        <CardDescription>{description}</CardDescription>
      </CardHeader>
      <CardContent>
        <div class="space-y-2">
          <For each={props.items}>
            {(item) => (
              <div class="flex gap-2 bg-neutral-800 p-3">
                <div class="flex-1 flex flex-col gap-2">
                  <NameSpace
                    namespace={item.namespace}
                    setNamespace={(ns) =>
                      props.updateItem(item.id, "namespace", ns)
                    }
                    rootToken={props.rootToken}
                    tokenRootNamespace={props.tokenRootNamespace}
                    getAllTokens={props.getAllTokens}
                  />
                  {showValueField && (
                    <TextField class="flex-1">
                      <TextFieldInput
                        value={item.value}
                        onInput={(e) =>
                          props.updateItem(
                            item.id,
                            "value",
                            e.currentTarget.value
                          )
                        }
                        placeholder={
                          props.type === "source"
                            ? "Prefix pattern (e.g. $y)"
                            : "Template value"
                        }
                        class="text-sm font-mono resize-none"
                      />
                    </TextField>
                  )}
                </div>
              </div>
            )}
          </For>
        </div>
      </CardContent>
    </Card>
  );
}
