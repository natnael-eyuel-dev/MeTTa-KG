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

interface TailsUnionProps {
  type: "patterns" | "templates";
  items: Item;
  addItem: () => void;
  updateItem: (id: string, field: "namespace", value: string[]) => void;
  accentColor: string;
  rootToken: boolean;
  tokenRootNamespace: () => string[];
  getAllTokens: () => Promise<Token[]>;
}

export function TailsUnionInput(props: TailsUnionProps) {
  const title = props.type === "patterns" ? "Patterns" : "Templates";
  const description =
    props.type === "patterns"
      ? "Define pattern to match against"
      : "Define template for operation";

  return (
    <Card class={`border-l-4 border-l-${props.accentColor}`}>
      <CardHeader>
        <div class="flex items-center">
          <div
            class={`w-2 h-2 bg-${props.accentColor} rounded-full mr-2`}
          ></div>
          <CardTitle>{title}</CardTitle>
        </div>
        <CardDescription>{description}</CardDescription>
      </CardHeader>
      <CardContent>
        <div class="space-y-2">
          <div class="flex gap-2 bg-neutral-800 p-3">
            <NameSpace
              namespace={props.items.namespace}
              setNamespace={(ns) =>
                props.updateItem(props.items.id, "namespace", ns)
              }
              rootToken={props.rootToken}
              tokenRootNamespace={props.tokenRootNamespace}
              getAllTokens={props.getAllTokens}
            />
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
