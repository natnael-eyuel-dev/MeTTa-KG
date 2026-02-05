import { Route, Router, useNavigate } from "@solidjs/router";
import { createSignal, For, onMount, Show, lazy } from "solid-js";
import CompositionPage from "../composition/Composition";
import IntersectionPage from "../intersection/Intersection";
import Sidebar from "~/pages/index/components/Sidebar";
import Header from "~/pages/index/components/Header";
import Upload from "lucide-solid/icons/upload";
import Database from "lucide-solid/icons/database";
import RotateCcw from "lucide-solid/icons/rotate-ccw";
import Download from "lucide-solid/icons/download";
import Key from "lucide-solid/icons/key";
import NotImplemented from "~/components/common/NotImplemented";
import Trash2 from "lucide-solid/icons/trash-2";
import CommandPalette from "~/components/common/CommandPalette";
import UnionPage from "../union/Union";
import { showToast } from "~/components/ui/Toast";
import { checkConfiguration, isConfigured } from "~/lib/state";

const LoadPage = lazy(() => import("../load/Load"));
const UploadPage = lazy(() => import("../upload/Upload"));
const TransformPage = lazy(() => import("../transform/Transform"));
const ExportPage = lazy(() => import("../export/Export"));
const TokensPage = lazy(() => import("../tokens/Tokens"));
const ClearPage = lazy(() => import("../clear/Clear"));
const LandingPage = lazy(() => import("../landing/Landing"));

export const sidebarSections = [
  {
    title: "Inspection and Visualization",
    items: [
      {
        id: "explore",
        label: "Explore",
        icon: Database,
        to: "/explore",
        component: LoadPage,
      },
      {
        id: "clear",
        label: "Clear",
        icon: Trash2,
        to: "/clear",
        component: ClearPage,
      },
    ],
  },
  {
    title: "Set and Algebraic Operations",
    items: [
      {
        id: "transform",
        label: "Transform",
        icon: RotateCcw,
        to: "/transform",
        component: TransformPage,
      },
      {
        id: "composition",
        label: "Composition",
        icon: () => <span class="text-xl">∪</span>,
        to: "/composition",
        component: CompositionPage,
      },
      {
        id: "union",
        label: "Union",
        icon: () => <span class="text-xl">∪</span>,
        to: "/union",
        component: UnionPage,
      },
      {
        id: "intersection",
        label: "Intersection",
        icon: () => <span class="text-xl">∩</span>,
        to: "/intersection",
        component: IntersectionPage,
      },
      {
        id: "difference",
        label: "Difference",
        icon: () => <span class="text-xl font-bold">∖</span>,
        to: "/difference",
      },
      {
        id: "restrict",
        label: "Restrict",
        icon: () => <span class="text-xl font-bold">◁</span>,
        to: "/restrict",
      },
      {
        id: "decapitate",
        label: "Decapitate",
        icon: () => <span class="text-xl">T</span>,
        to: "/decapitate",
      },
      {
        id: "head",
        label: "Head",
        icon: () => <span class="text-xl">H</span>,
        to: "/head",
      },
      {
        id: "cartesian",
        label: "Cartesian",
        icon: () => <span class="text-xl">X</span>,
        to: "/cartesian",
      },
    ],
  },
  {
    title: "Utility",
    items: [
      {
        id: "upload",
        label: "Import",
        icon: Upload,
        to: "/upload",
        component: UploadPage,
      },
      {
        id: "export",
        label: "Export",
        icon: Download,
        to: "/export",
        component: ExportPage,
      },
      {
        id: "tokens",
        label: "Tokens",
        icon: Key,
        to: "/tokens",
        component: TokensPage,
      },
    ],
  },
];

const AppLayout = (
  props: any /* eslint-disable-line @typescript-eslint/no-explicit-any */
) => {
  const [activeTab, setActiveTab] = createSignal("explore");
  const [isChecked, setIsChecked] = createSignal(false);
  const navigate = useNavigate();

  onMount(async () => {
    const configured = await checkConfiguration();
    setIsChecked(true);

    if (!configured) {
      showToast({
        title: "Configuration Required",
        description: "Please configure the database and server settings.",
        variant: "destructive",
        duration: 5000,
      });
      navigate("/", { replace: true });
    }
  });

  return (
    <>
      <Show when={isChecked() && isConfigured()}>
        <CommandPalette />
        <div class="w-full h-screen flex ">
          <div class="flex h-full">
            <Sidebar
              activeTab={activeTab}
              setActiveTab={setActiveTab}
              sidebarSections={sidebarSections}
            />
          </div>

          <div class="w-full h-full flex flex-col">
            <Header />

            <div class="flex-1 w-full pl-4 pt-2 overflow-y-scroll">
              {props.children}
            </div>
          </div>
        </div>
      </Show>
    </>
  );
};

const NotImplementedWrapper = (name: string) => () => (
  <NotImplemented name={name} />
);

const App = () => {
  return (
    <div class="flex">
      <div class="flex-1 flex flex-col">
        <Router>
          <Route path="/" component={LandingPage} />
          <Route path="*" component={AppLayout}>
            <For each={sidebarSections}>
              {(section) => (
                <For each={section.items}>
                  {(item) => (
                    <Route
                      path={item.to}
                      component={
                        item.component || NotImplementedWrapper(item.label)
                      }
                    />
                  )}
                </For>
              )}
            </For>
          </Route>
        </Router>
      </div>
    </div>
  );
};

export default App;
