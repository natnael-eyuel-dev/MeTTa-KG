# MeTTa-KG

https://deepfunding.ai/proposal/scalable-metta-knowledge-graphs/

This README is WIP and is subject to change.

## Spaces

A Knowledge Graph (KG) corresponds to a hierarchy of spaces. Each space has a name, which we refer to as its namespace. The root space is identified by the "/" namespace, while its direct subspaces (spaces on the second level of the hierachy) are identified by namespaces such as "/subspace1/", and so on.

Namespaces are similar to filesystem paths, especially when viewed as a tree. In this view, the difference is that for namespaces, interior nodes play the same role as leaf nodes, while their roles differ in the case of a filesystem path. In particular, every namespace in the hierachy identifies a space. Moreover, namespaces are never "terminal", meaning that it is always possible to embed subspaces further.

Spaces support two operations, read and write. As the name suggests, a read operation on a space provides a read-only view of that space. A write operation works by applying some transformations to the space, creating a new space in the process and preserving the original space. The newly created space is embedded into the space the write operation was used on.

### Namespace Rules

A namespace should:

- start with '/'
- end with '/'
- consist of segments separated by '/' that:
  - contain only alphanumeric characters, '-', '\_'
  - start with an alphanumeric character
  - end with an alphanumeric character

## Tokens

Tokens give access to spaces in the KG by linking to their namespaces. A token has a number of associated permissions:

- `read`: allow read-only viewing of the space
- `write`: allow write operations on the space
- `share-read`: allow creation of a new token with the 'read' permission on the same space
- `share-write`: allow creation of a new token with the 'write' permission on the same space

There exists a single "admin" token. It is associated with the root namespace `/` and has a special permission named `share-share`. The root token can be refreshed (= regenerated), but can not be deleted.

> [!WARNING]
> Operations are **recursive**. For example, tokens with the `write` permission for a namespace `/space/` can be used to write in `/space/subspace/`, `/space/subspace/another-subspace/`.

Existing tokens can be used to create new ones, provided they have any of the `share` permissions listed above.

> [!WARNING]
> Deleting a token also deletes any tokens that were created from it, recursively.

It is currently not possible to modify an existing token's namespace, description, or permissions. Tokens can be refreshed if leaked by accident.

Tokens are managed on the `/tokens` page ([Demo](https://metta-kg.vercel.app/tokens)).

## Editor

The editor allows you to interact with the contents of the KG using the [MeTTa](https://metta-lang.dev/) language.

The editor can be found on the `/` page ([Demo](https://metta-kg.vercel.app/)).

## Translations

Documentation on translations can be found [here](./translations/README.md).

## Demo

A live version of MeTTa-KG can be found at: https://metta-kg.vercel.app.

# MeTTa-KG (Single-Binary Distribution)

Fast, portable, low-friction runtime for MeTTa-KG. Ships as a single binary that embeds the web UI and exposes a clap-based CLI.

## Install (one-liner)

- macOS/Linux (curl):
  curl -LSfs https://github.com/arist76/MeTTa-KG/releases/latest/download/install.sh | sh

- Windows (PowerShell):
  iwr https://github.com/arist76/MeTTa-KG/releases/latest/download/install.ps1 -useb | iex

Installer will place `metta-kg` in ~/.cargo/bin or a platform-appropriate location.

## Run (single command)

metta-kg --address 127.0.0.1 --port 3030

By default a browser window is opened at http://127.0.0.1:3030 (use --no-browser to disable).

Flags:
- --address <ip> (default 127.0.0.1)
- --port <port> (default 3030)
- --base-path <path> (default /)
- --no-browser

Subcommands:
- run (default)
- import (reserved)
- query (reserved)

Environment variables:
- METTA_KG_DATABASE_URL (optional; default for local binary: postgresql://metta-kg-admin:password123@localhost:5432/metta-kg)
- METTA_KG_SECRET
- METTA_KG_MORK_URL

Database URL selection:
- Local binary: uses METTA_KG_DATABASE_URL if set, otherwise defaults to postgresql://metta-kg-admin:password123@localhost:5432/metta-kg
- Docker: auto-detected via DOCKER=1 or /.dockerenv and uses postgresql://metta-kg-admin:password123@db:5432/metta-kg unless METTA_KG_DATABASE_URL is set

## Binary vs Docker workflows

Preferred: single binary
- Build locally: (requires Rust and Node)
  cd api && cargo run --release -- --address 127.0.0.1 --port 3030

Docker (development)
- docker build -t metta-kg:dev .
- docker compose up

The runtime container only ships the binary (and translations venv where needed).

## Packaging and Releases

- cargo-dist builds binaries for macOS, Linux, Windows with installers and updater.
- Tag a release (vX.Y.Z) and GitHub Actions will build and publish artifacts.

## Nix (optional)

nix build

This will build frontend and produce a deterministic metta-kg binary.
