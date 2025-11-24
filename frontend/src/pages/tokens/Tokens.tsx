import { Component, Show, createSignal } from "solid-js";
import { CommandCard } from "~/components/common/CommandCard";
import { RootTokenForm } from "./components/RootTokenForm";
import { CreateTokenForm } from "./components/CreateTokenForm";
import { TokenTable } from "./components/TokenTable";
import { Button } from "~/components/ui/Button";
import { rootToken, setRootToken, formatedNamespace } from "~/lib/state";

import {
  TextField,
  TextFieldInput,
  TextFieldLabel,
} from "~/components/ui/TextField";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "~/components/ui/Dialog";
import {
  tokens,
  mutateTokens,
  filteredAndSortedTokens,
  selectedTokens,
  handleRefresh,
  handleDelete,
  setNamespaceFilter,
  setDescriptionFilter,
  sortColumn,
  sortDirection,
  handleSort,
  handleSelectAll,
  handleSelectToken,
  isTokenSelected,
} from "./lib";

export const TokensPage: Component = () => {
  // Add state to control dialog visibility
  const [isRefreshOpen, setIsRefreshOpen] = createSignal(false);
  const [isDeleteOpen, setIsDeleteOpen] = createSignal(false);

  const isRootTokenSelected = () =>
    selectedTokens().some((t) => t.code === rootToken());

  const isCurrentNamespaceTokenSelected = () => {
    const currentNs = formatedNamespace();
    const trimTrailingSlash = (str: string) =>
      str !== "/" && str.endsWith("/") ? str.slice(0, -1) : str;
    return selectedTokens().some((t) => {
      const tokenNs = t.namespace.startsWith("/")
        ? t.namespace
        : "/" + t.namespace;
      return currentNs === trimTrailingSlash(tokenNs);
    });
  };
  return (
    <div class="ml-10 mt-8 space-y-8">
      <CommandCard
        title="Token Management"
        description="Manage access by creating, viewing, and revoking tokens."
      >
        <RootTokenForm initialToken={rootToken()} onLoad={setRootToken} />
      </CommandCard>

      <Show when={rootToken()}>
        <CommandCard
          title="Create New Token"
          description="Define a namespace, description, and permissions for a new token."
        >
          <CreateTokenForm
            rootToken={rootToken()!}
            tokens={tokens() || []}
            onTokenCreated={(token) =>
              mutateTokens((prev) => [...(prev || []), token])
            }
          />
        </CommandCard>

        <CommandCard
          title="Existing Tokens"
          description="View and manage your current tokens."
        >
          <div class="space-y-4">
            {/* ✅ Combined row with filters and buttons */}
            <div class="flex gap-4 items-end">
              <TextField class="flex-1">
                <TextFieldLabel>Filter by Namespace</TextFieldLabel>
                <TextFieldInput
                  placeholder="Enter namespace..."
                  onInput={(e) => setNamespaceFilter(e.currentTarget.value)}
                />
              </TextField>
              <TextField class="flex-1">
                <TextFieldLabel>Filter by Description</TextFieldLabel>
                <TextFieldInput
                  placeholder="Enter description..."
                  onInput={(e) => setDescriptionFilter(e.currentTarget.value)}
                />
              </TextField>

              {/* Buttons aligned to bottom */}
              <div class="flex gap-2">
                <Dialog open={isRefreshOpen()} onOpenChange={setIsRefreshOpen}>
                  <DialogTrigger
                    as={Button}
                    disabled={
                      selectedTokens().length === 0 ||
                      isRootTokenSelected() ||
                      isCurrentNamespaceTokenSelected()
                    }
                  >
                    Refresh ({selectedTokens().length})
                  </DialogTrigger>
                  <DialogContent>
                    <DialogHeader>
                      <DialogTitle>Refresh Tokens</DialogTitle>
                      <DialogDescription>
                        Are you sure you want to refresh{" "}
                        {selectedTokens().length} token(s)? This will generate
                        new codes for the selected tokens.
                      </DialogDescription>
                    </DialogHeader>
                    <DialogFooter>
                      <Button
                        variant="outline"
                        onClick={() => setIsRefreshOpen(false)}
                      >
                        Cancel
                      </Button>
                      <Button
                        onClick={async () => {
                          await handleRefresh();
                          setIsRefreshOpen(false);
                        }}
                      >
                        Refresh
                      </Button>
                    </DialogFooter>
                  </DialogContent>
                </Dialog>

                <Dialog open={isDeleteOpen()} onOpenChange={setIsDeleteOpen}>
                  <DialogTrigger
                    as={Button}
                    disabled={
                      selectedTokens().length === 0 ||
                      isRootTokenSelected() ||
                      isCurrentNamespaceTokenSelected()
                    }
                    variant="destructive"
                  >
                    Delete ({selectedTokens().length})
                  </DialogTrigger>
                  <DialogContent>
                    <DialogHeader>
                      <DialogTitle>Delete Tokens</DialogTitle>
                      <DialogDescription>
                        Are you sure you want to delete{" "}
                        {selectedTokens().length} token(s)?
                        <div class="mt-2 p-2 bg-destructive/10 text-destructive rounded-md font-medium">
                          Warning: This will recursively delete all child tokens
                          (sub-namespaces) associated with these tokens.
                        </div>
                        <div class="mt-2">This action cannot be undone.</div>
                      </DialogDescription>
                    </DialogHeader>
                    <DialogFooter>
                      <Button
                        variant="outline"
                        onClick={() => setIsDeleteOpen(false)}
                      >
                        Cancel
                      </Button>
                      <Button
                        variant="destructive"
                        onClick={async () => {
                          await handleDelete();
                          setIsDeleteOpen(false);
                        }}
                      >
                        Delete
                      </Button>
                    </DialogFooter>
                  </DialogContent>
                </Dialog>
              </div>
            </div>

            <TokenTable
              tokens={filteredAndSortedTokens()}
              sortState={{ column: sortColumn, direction: sortDirection }}
              onSort={handleSort}
              onSelectAll={handleSelectAll}
              onSelectToken={handleSelectToken}
              isTokenSelected={isTokenSelected}
            />
          </div>
        </CommandCard>
      </Show>
    </div>
  );
};

export default TokensPage;
