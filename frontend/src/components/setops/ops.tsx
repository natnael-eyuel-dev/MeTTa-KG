import { createMemo } from "solid-js";
import { createSetOpState } from "~/components/setops/state";
import { ItemList, Item, ItemListProps } from "~/components/setops/ItemList";
import { ExpressionPreview } from "~/components/setops/ExpressionPreview";
import { Button } from "~/components/ui/Button";

export interface SideConfig {
  count: number;
  min?: number;
  showValue: boolean;
  allowAdd?: boolean;
  disableAdd?: boolean;
  labels?: (index: number) => string;
}

export interface OperationConfig {
  id: string;
  title?: string;
  description?: string;
  note?: string;
  patterns: SideConfig;
  templates: SideConfig;
  namespaceDefaults?: string[];
  composeExpression: (patterns: Item[], templates: Item[]) => string;
  canRun: (patterns: Item[], templates: Item[]) => boolean;
}

export function createOperationConfig(
  partial: Partial<OperationConfig> &
    Pick<
      OperationConfig,
      "id" | "patterns" | "templates" | "composeExpression" | "canRun"
    >
): OperationConfig {
  return {
    description: partial.description || "",
    note: partial.note || "",
    namespaceDefaults: partial.namespaceDefaults || [""],
    ...partial,
  };
}

export function formatNamespace(parts: string[], value?: string): string {
  const segs = parts.slice(1).filter(Boolean);
  const raw = (value || "").trim();
  const wrap = (v: string) => (v.startsWith("(") ? v : `(${v})`);
  if (segs.length === 0) return raw || "()";
  if (segs.length === 1)
    return raw ? `(${segs[0]} ${wrap(raw)})` : `(${segs[0]})`;
  if (!raw && segs.length === 2) return `${segs[0]} (${segs[1]})`;
  let inner = segs[segs.length - 1];
  if (raw) inner = `(${inner} ${wrap(raw)})`;
  for (let i = segs.length - 2; i >= 1; i--) inner = `(${segs[i]} ${inner})`;
  return `${segs[0]} ${inner}`;
}

export function createOpsContext(config: OperationConfig) {
  const state = createSetOpState({
    patterns: {
      initial: config.patterns.count,
      showValue: config.patterns.showValue,
      min: config.patterns.min,
      allowAdd: config.patterns.allowAdd,
    },
    templates: {
      initial: config.templates.count,
      showValue: config.templates.showValue,
      min: config.templates.min,
      allowAdd: config.templates.allowAdd,
    },
    compose: config.composeExpression,
    canRun: config.canRun,
    defaultNamespace: config.namespaceDefaults,
  });
  const expr = () => state.expression();
  const runnable = createMemo(() => state.runnable());
  return {
    config,
    get patterns() {
      return state.patterns;
    },
    get templates() {
      return state.templates;
    },
    addPattern: state.addPattern,
    removePattern: state.removePattern,
    updatePatternNamespace: state.updatePatternNamespace,
    updatePatternValue: state.updatePatternValue,
    addTemplate: state.addTemplate,
    removeTemplate: state.removeTemplate,
    updateTemplateNamespace: state.updateTemplateNamespace,
    updateTemplateValue: state.updateTemplateValue,
    expression: expr,
    runnable,
  };
}

export function OpsColumn(props: {
  side: "patterns" | "templates";
  ctx: ReturnType<typeof createOpsContext>;
  rootToken: boolean;
  tokenRootNamespace: () => string[];
  getAllTokens: () => Promise<{ namespace: string; description: string }[]>;
}) {
  const { side, ctx } = props;
  const cfg = ctx.config[side];
  const items = side === "patterns" ? ctx.patterns : ctx.templates;
  const add = side === "patterns" ? ctx.addPattern : ctx.addTemplate;
  const remove = side === "patterns" ? ctx.removePattern : ctx.removeTemplate;
  const updateNs =
    side === "patterns"
      ? ctx.updatePatternNamespace
      : ctx.updateTemplateNamespace;
  const updateVal =
    side === "patterns" ? ctx.updatePatternValue : ctx.updateTemplateValue;
  return (
    <ItemList
      title={side === "patterns" ? "Patterns" : "Templates"}
      description={
        side === "patterns"
          ? "Configure pattern namespaces / values"
          : "Configure template namespaces / values"
      }
      items={items}
      onAdd={add}
      onRemove={remove}
      onUpdateNamespace={updateNs}
      onUpdateValue={updateVal}
      showValue={cfg.showValue}
      minItems={cfg.min}
      disableAdd={cfg.disableAdd}
      itemLabels={cfg.labels}
      rootToken={props.rootToken}
      tokenRootNamespace={props.tokenRootNamespace}
      getAllTokens={props.getAllTokens}
    />
  );
}

export function OpsExpressionPreview(props: {
  ctx: ReturnType<typeof createOpsContext>;
}) {
  const { ctx } = props;
  return (
    <ExpressionPreview
      description={ctx.config.description}
      note={ctx.config.note}
      expressionSource={ctx.expression}
    />
  );
}

export function OpsActionBar(props: {
  ctx: ReturnType<typeof createOpsContext>;
  isLoading: boolean;
  isPolling: boolean;
  onRun: (
    patterns: ReturnType<typeof createOpsContext>["patterns"],
    templates: ReturnType<typeof createOpsContext>["templates"],
    expression: string
  ) => void;
  idleLabel: string;
  runningLabel?: string;
  pollingLabel?: string;
}) {
  const runnable = () =>
    props.ctx.runnable() && !props.isLoading && !props.isPolling;
  const handleRun = () => {
    if (!runnable()) return;
    props.onRun(
      props.ctx.patterns,
      props.ctx.templates,
      props.ctx.expression()
    );
  };
  return (
    <div class="flex justify-end">
      <Button
        onClick={handleRun}
        disabled={!runnable()}
        class="inline-flex items-center justify-center w-[200px] h-10"
      >
        {props.isLoading || props.isPolling ? (
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
        ) : null}
        {props.isLoading
          ? props.runningLabel || "Processing..."
          : props.isPolling
            ? props.pollingLabel || "Waiting..."
            : props.idleLabel}
      </Button>
    </div>
  );
}

export type { Item as SetOpItem, ItemListProps as SetOpItemListProps };
