import { createSignal, onMount } from "solid-js";
import { useNavigate } from "@solidjs/router";
import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/Card";
import {
  TextField,
  TextFieldInput,
  TextFieldLabel,
} from "~/components/ui/TextField";
import { Button } from "~/components/ui/Button";
import { showToast } from "~/components/ui/Toast";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "~/components/ui/Dialog";
import Database from "lucide-solid/icons/database";
import Server from "lucide-solid/icons/server";
import ArrowRight from "lucide-solid/icons/arrow-right";
import Box from "lucide-solid/icons/box";
import Key from "lucide-solid/icons/key";
import Code from "lucide-solid/icons/code";
import { checkConfiguration } from "~/lib/state";
import previewImage from "~/assets/preview.png";

export default function Landing() {
  const [dbUrl, setDbUrl] = createSignal("");
  const [morkUrl, setMorkUrl] = createSignal("http://127.0.0.1:8001");
  const [isLoading, setIsLoading] = createSignal(false);
  const [isConfigOpen, setIsConfigOpen] = createSignal(false);
  const [dbType, setDbType] = createSignal<"sqlite" | "postgres">("sqlite"); // Default to sqlite
  const navigate = useNavigate();

  onMount(async () => {
    const configured = await checkConfiguration();
    if (configured) {
      navigate("/explore", { replace: true });
    } else {
      try {
        const res = await fetch("/build-info");
        if (res.ok) {
          const data = await res.json();
          setDbType(data.db_type);
        }
      } catch (e) {
        console.error("Failed to fetch build info", e);
      }
    }
  });

  const handleSubmit = async (e: Event) => {
    e.preventDefault();

    if (!dbUrl().trim()) {
      showToast({
        title: "Validation Error",
        description: "Database URL is required.",
        variant: "destructive",
      });
      return;
    }

    if (!morkUrl().trim()) {
      showToast({
        title: "Validation Error",
        description: "MORK Server URL is required.",
        variant: "destructive",
      });
      return;
    }

    setIsLoading(true);

    try {
      const formData = new FormData();
      formData.append("database_url", dbUrl());
      formData.append("mork_server_url", morkUrl());
      formData.append("mettakg_api_url", "http://127.0.0.1:8000");

      const response = await fetch("/submit", {
        method: "POST",
        body: formData,
      });

      if (response.ok || response.redirected) {
        showToast({
          title: "Configuration Saved",
          description: "Server is restarting with new settings...",
          variant: "default",
        });

        setTimeout(() => {
          window.location.href = "/explore";
        }, 2000);
      } else {
        throw new Error("Submission failed");
      }
    } catch (error) {
      console.error(error);
      showToast({
        title: "Connection Error",
        description: "Failed to connect to the configuration server.",
        variant: "destructive",
      });
      setIsLoading(false);
    }
  };

  return (
    <div class="min-h-screen w-full bg-background text-foreground flex flex-col">
      {/* Hero Section */}
      <header class="w-full border-b border-border/40 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 sticky top-0 z-50">
        <div class="container flex h-14 max-w-screen-2xl items-center justify-between px-8">
          <div class="flex items-center gap-2 font-bold text-xl">
            <div class="w-8 h-8 bg-primary rounded-md flex items-center justify-center text-primary-foreground">
              M
            </div>
            MeTTa-KG
          </div>
          <nav class="flex items-center gap-4">
            <a
              href="https://github.com/trueagi-io/metta-kg"
              target="_blank"
              class="text-sm font-medium text-muted-foreground hover:text-primary transition-colors"
            >
              GitHub
            </a>
            <a
              href="https://metta-lang.dev/"
              target="_blank"
              class="text-sm font-medium text-muted-foreground hover:text-primary transition-colors"
            >
              MeTTa Lang
            </a>
          </nav>
        </div>
      </header>

      <main class="flex-1">
        <section class="space-y-6 pb-8 pt-6 md:pb-12 md:pt-10 lg:py-32">
          <div class="container flex max-w-[64rem] flex-col items-center gap-4 text-center mx-auto px-4">
            <h1 class="font-heading text-3xl sm:text-5xl md:text-6xl lg:text-7xl font-bold tracking-tight">
              Scalable MeTTa <br />
              <span class="text-primary">Knowledge Graphs</span>
            </h1>
            <p class="max-w-[42rem] leading-normal text-muted-foreground sm:text-xl sm:leading-8">
              A high-performance platform for managing hierarchical knowledge
              spaces, executing algebraic operations, and visualizing complex
              data structures using the MeTTa language.
            </p>
            <div class="space-x-4 pt-4">
              <Button
                size="lg"
                class="h-12 px-8 text-lg gap-2 shadow-lg shadow-primary/20"
                onClick={() => setIsConfigOpen(true)}
              >
                Get Started <ArrowRight class="w-5 h-5" />
              </Button>
              <Button
                variant="outline"
                size="lg"
                class="h-12 px-8 text-lg"
                as="a"
                href="https://deepfunding.ai/proposal/scalable-metta-knowledge-graphs/"
                target="_blank"
              >
                Learn More
              </Button>
            </div>
          </div>
        </section>

        {/* Feature Grid */}
        <section class="container space-y-6 bg-slate-50/50 dark:bg-transparent py-8 md:py-12 lg:py-24 mx-auto px-4">
          <div class="mx-auto grid justify-center gap-4 sm:grid-cols-2 md:max-w-[64rem] md:grid-cols-3">
            <Card class="bg-background border-border/50 transition-all hover:border-primary/50 hover:shadow-md">
              <CardHeader>
                <Box class="w-10 h-10 text-primary mb-2" />
                <CardTitle>Hierarchical Spaces</CardTitle>
                <CardDescription>
                  Organize knowledge in nested namespaces like a filesystem.
                  Support for read/write operations with strict embedding rules.
                </CardDescription>
              </CardHeader>
            </Card>
            <Card class="bg-background border-border/50 transition-all hover:border-primary/50 hover:shadow-md">
              <CardHeader>
                <Key class="w-10 h-10 text-primary mb-2" />
                <CardTitle>Secure Tokens</CardTitle>
                <CardDescription>
                  Granular access control with recursive permissions. Manage
                  read, write, and sharing capabilities across your graph.
                </CardDescription>
              </CardHeader>
            </Card>
            <Card class="bg-background border-border/50 transition-all hover:border-primary/50 hover:shadow-md">
              <CardHeader>
                <Code class="w-10 h-10 text-primary mb-2" />
                <CardTitle>MeTTa Editor</CardTitle>
                <CardDescription>
                  Interact directly with your Knowledge Graph using the MeTTa
                  language. Visualize results and debug complex queries.
                </CardDescription>
              </CardHeader>
            </Card>
          </div>
        </section>

        {/* Preview Image Section */}
        <section class="container py-8 md:py-12 lg:py-24 mx-auto px-4">
          <div class="mx-auto max-w-[64rem] space-y-4 text-center">
            <h2 class="font-heading text-3xl leading-[1.1] sm:text-3xl md:text-6xl font-bold">
              Powerful Visualization
            </h2>
            <p class="leading-normal text-muted-foreground sm:text-lg sm:leading-7 max-w-[42rem] mx-auto">
              Inspect your data with our advanced code explorer and graph
              visualization tools.
            </p>
          </div>
          <div class="mx-auto mt-10 max-w-[64rem] overflow-hidden rounded-xl border border-border bg-background shadow-2xl">
            <div class="aspect-video w-full bg-neutral-900 relative group">
              <div class="absolute inset-0 bg-gradient-to-t from-background to-transparent opacity-20 pointer-events-none" />
              <img
                src={previewImage}
                alt="MeTTa-KG Interface Preview"
                class="w-full h-full object-cover object-top opacity-90 transition-opacity group-hover:opacity-100"
                onError={(e) => {
                  e.currentTarget.style.display = "none";
                  e.currentTarget.parentElement!.classList.add(
                    "flex",
                    "items-center",
                    "justify-center",
                    "text-muted-foreground"
                  );
                  e.currentTarget.parentElement!.innerText =
                    "Interface Preview";
                }}
              />
            </div>
          </div>
        </section>
      </main>

      <footer class="py-6 md:px-8 md:py-0 border-t border-border/40">
        <div class="container flex flex-col items-center justify-between gap-4 md:h-24 md:flex-row mx-auto">
          <p class="text-center text-sm leading-loose text-muted-foreground md:text-left">
            Built by{" "}
            <a
              href="https://trueagi.io"
              target="_blank"
              class="font-medium underline underline-offset-4"
            >
              TrueAGI
            </a>
            . The source code is available on{" "}
            <a
              href="https://github.com/trueagi-io/metta-kg"
              target="_blank"
              class="font-medium underline underline-offset-4"
            >
              GitHub
            </a>
            .
          </p>
        </div>
      </footer>

      {/* Configuration Dialog */}
      <Dialog open={isConfigOpen()} onOpenChange={setIsConfigOpen}>
        <DialogContent class="sm:max-w-[525px]">
          <DialogHeader>
            <DialogTitle>Configure Server</DialogTitle>
            <DialogDescription>
              Enter your database and MORK server details to initialize the
              platform.
            </DialogDescription>
          </DialogHeader>
          <form onSubmit={handleSubmit} class="space-y-4 py-4">
            <div class="space-y-2">
              <TextField value={dbUrl()} onChange={setDbUrl}>
                <TextFieldLabel class="flex items-center gap-2">
                  <Database class="w-4 h-4" /> Database URL
                </TextFieldLabel>
                <TextFieldInput
                  placeholder={
                    dbType() === "sqlite"
                      ? "metta_kg.db"
                      : "postgres://user:pass@localhost/dbname"
                  }
                  disabled={isLoading()}
                />
              </TextField>
              <p class="text-[0.8rem] text-muted-foreground">
                {dbType() === "sqlite"
                  ? "Path to SQLite file (will be created if missing)."
                  : "PostgreSQL connection string."}
              </p>
            </div>

            <div class="space-y-2">
              <TextField value={morkUrl()} onChange={setMorkUrl}>
                <TextFieldLabel class="flex items-center gap-2">
                  <Server class="w-4 h-4" /> MORK Server URL
                </TextFieldLabel>
                <TextFieldInput
                  placeholder="http://127.0.0.1:8001"
                  disabled={isLoading()}
                />
              </TextField>
            </div>

            <DialogFooter>
              <Button type="submit" class="w-full" disabled={isLoading()}>
                {isLoading() ? "Configuring..." : "Start Server"}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </div>
  );
}
