import { Component, Show, createSignal, onCleanup } from "solid-js";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "~/components/ui/Card";
import { Copy, Check } from "lucide-solid";
import { showToast } from "~/components/ui/Toast";

interface PreviewProps {
  description?: string;
  note?: string;
  expressionSource: () => string;
}

export const ExpressionPreview: Component<PreviewProps> = (props) => {
  const [copied, setCopied] = createSignal(false);
  let timer: number | undefined;

  const expression = () => props.expressionSource() || "";

  const copy = () => {
    if (!navigator?.clipboard) return;
    navigator.clipboard.writeText(expression()).catch(() => {});
    setCopied(true);
    showToast({
      title: "Copied",
      description: "Expression copied to clipboard",
    });
    timer = window.setTimeout(() => setCopied(false), 2000);
  };

  onCleanup(() => {
    if (timer) clearTimeout(timer);
  });

  return (
    <Card class="sticky top-4 relative mb-4">
      <CardHeader class="pr-10">
        <CardTitle>S-Expression Preview</CardTitle>
        <Show when={props.description}>
          <CardDescription>{props.description}</CardDescription>
        </Show>
        <button
          type="button"
          onClick={copy}
          class="absolute top-3 right-3 p-2 rounded hover:bg-accent transition-colors"
          aria-label="Copy expression"
        >
          {copied() ? <Check class="w-4 h-4" /> : <Copy class="w-4 h-4" />}
        </button>
      </CardHeader>
      <CardContent>
        <pre class="text-sm font-mono bg-muted p-3 rounded overflow-auto min-h-[90px]">
          {expression()}
        </pre>
        <Show when={props.note}>
          <div class="mt-4 p-3 bg-muted/50 rounded text-sm text-muted-foreground">
            {props.note}
          </div>
        </Show>
      </CardContent>
    </Card>
  );
};
