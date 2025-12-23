import { createStore, produce } from "solid-js/store";
import { createUniqueId } from "solid-js";
import type { Item } from "~/components/setops/ItemList";

interface ConfigSide {
  initial: number;
  showValue: boolean;
  min?: number;
  allowAdd?: boolean;
}

interface Config {
  patterns: ConfigSide;
  templates: ConfigSide;
  compose?: (patterns: Item[], templates: Item[]) => string;
  canRun?: (patterns: Item[], templates: Item[]) => boolean;
  defaultNamespace?: string[];
}

export interface StateApi {
  patterns: Item[];
  templates: Item[];
  addPattern?: () => void;
  removePattern?: (id: string) => void;
  updatePatternNamespace: (id: string, ns: string[]) => void;
  updatePatternValue: (id: string, v: string) => void;
  addTemplate?: () => void;
  removeTemplate?: (id: string) => void;
  updateTemplateNamespace: (id: string, ns: string[]) => void;
  updateTemplateValue: (id: string, v: string) => void;
  expression: () => string;
  runnable: () => boolean;
  config: Config;
}

function wrapValue(v: string): string {
  return v.startsWith("(") ? v : `(${v})`;
}

function formatDefaultItem(item: Item): string {
  const parts = (item.namespace || []).slice(1).filter(Boolean);
  const val = (item.value || "").trim();
  if (parts.length === 0) return val || "()";
  if (parts.length === 1)
    return val ? `${parts[0]} ${wrapValue(val)}` : parts[0];
  let inner = parts[parts.length - 1];
  if (val) inner = `(${inner} ${wrapValue(val)})`;
  for (let i = parts.length - 2; i >= 1; i--) inner = `(${parts[i]} ${inner})`;
  return `${parts[0]} ${inner}`;
}

function defaultCompose(patterns: Item[], templates: Item[]): string {
  const patt = patterns.map(formatDefaultItem).join(" ");
  const templ = templates.map(formatDefaultItem).join(" ");
  return `(transform\n    (, ${patt})\n    (, ${templ})\n)`;
}

function createSeededItem(nsSeed: string[]): Item {
  return { id: createUniqueId(), namespace: [...nsSeed], value: "" };
}

function pushItem(list: Item[], nsSeed: string[]): void {
  list.push(createSeededItem(nsSeed));
}

function spliceItem(list: Item[], id: string, min: number): void {
  if (list.length <= min) return;
  const idx = list.findIndex((i) => i.id === id);
  if (idx >= 0) list.splice(idx, 1);
}

function updateNamespace(list: Item[], id: string, ns: string[]): void {
  const item = list.find((i) => i.id === id);
  if (item) item.namespace = ns;
}

function updateValue(list: Item[], id: string, v: string): void {
  const item = list.find((i) => i.id === id);
  if (item) item.value = v;
}

function makePush(nsSeed: string[]) {
  return (list: Item[]) => pushItem(list, nsSeed);
}
function makeSplice(id: string, min: number) {
  return (list: Item[]) => spliceItem(list, id, min);
}
function makeUpdateNs(id: string, ns: string[]) {
  return (list: Item[]) => updateNamespace(list, id, ns);
}
function makeUpdateVal(id: string, v: string) {
  return (list: Item[]) => updateValue(list, id, v);
}

function computeExpression(
  config: Config,
  patterns: Item[],
  templates: Item[]
) {
  return config.compose
    ? config.compose(patterns, templates)
    : defaultCompose(patterns, templates);
}

function isRunnable(config: Config, patterns: Item[], templates: Item[]) {
  if (config.canRun) return config.canRun(patterns, templates);
  const pattOk =
    !config.patterns.showValue || patterns.some((p) => (p.value || "").trim());
  const templOk =
    !config.templates.showValue ||
    templates.some((t) => (t.value || "").trim());
  return pattOk && templOk;
}

export function createSetOpState(config: Config): StateApi {
  const nsSeed = config.defaultNamespace || [""];
  const [state, setState] = createStore({
    patterns: Array.from({ length: config.patterns.initial }, () =>
      createSeededItem(nsSeed)
    ),
    templates: Array.from({ length: config.templates.initial }, () =>
      createSeededItem(nsSeed)
    ),
  });

  const addPattern = config.patterns.allowAdd
    ? () => setState("patterns", produce(makePush(nsSeed)))
    : undefined;
  const removePattern = config.patterns.allowAdd
    ? (id: string) =>
        setState("patterns", produce(makeSplice(id, config.patterns.min || 1)))
    : undefined;
  const updatePatternNamespace = (id: string, ns: string[]) =>
    setState("patterns", produce(makeUpdateNs(id, ns)));
  const updatePatternValue = (id: string, v: string) =>
    setState("patterns", produce(makeUpdateVal(id, v)));

  const addTemplate = config.templates.allowAdd
    ? () => setState("templates", produce(makePush(nsSeed)))
    : undefined;
  const removeTemplate = config.templates.allowAdd
    ? (id: string) =>
        setState(
          "templates",
          produce(makeSplice(id, config.templates.min || 1))
        )
    : undefined;
  const updateTemplateNamespace = (id: string, ns: string[]) =>
    setState("templates", produce(makeUpdateNs(id, ns)));
  const updateTemplateValue = (id: string, v: string) =>
    setState("templates", produce(makeUpdateVal(id, v)));

  const expression = () =>
    computeExpression(config, state.patterns, state.templates);
  const runnable = () => isRunnable(config, state.patterns, state.templates);

  return {
    get patterns() {
      return state.patterns;
    },
    get templates() {
      return state.templates;
    },
    addPattern,
    removePattern,
    updatePatternNamespace,
    updatePatternValue,
    addTemplate,
    removeTemplate,
    updateTemplateNamespace,
    updateTemplateValue,
    expression,
    runnable,
    config,
  } as StateApi;
}

export type { Config as SetOpConfig };
