<!-- Generated from tmtroot/readme.tmt. Edit that, then `tomet export .`. -->

# Tomet Wiki

Wiki system for the Tomet ecosystem.

## Overview

`tomet-wiki` is a Wiki engine and static site generator designed for Tomet documents, featuring inline editing support with conflict resolution and Git CMS integration.

## Usage

```bash
tomet-wiki init [dir] [--title <title>] [--owner <owner>] [--repo <repo>]
tomet-wiki build [dir] [-d/--dest <dest>] [--strict]
tomet-wiki serve [dir] [-d/--dest <dest>] [--host <ip>] [-p/--port <port>]
```

## Configuration (`wiki.toml`)

A `wiki.toml` configuration file in the vault root controls site generation and CMS authentication:

```toml
[wiki]
title = "My Wiki"
lang = "ja"
dest = "dist"

[cms]
owner = "my-org"
repo = "my-wiki"
branch = "main"
auth_endpoint = "https://auth.example.com/oauth"

[build]
url_prefix = "/wiki"
exclude = [
  "00-09 System/01 Apps"
]
```

## Workspace Layout

- `crates/tomet-wiki-headless` (`tomet-wiki-headless`) -- Headless Wiki engine composing `tomet-shelf-*` component crates. Owns `WikiConfig`, `build_wiki`, and `WikiDevHandler`.
- `crates/tomet-wiki-assets` (`tomet-wiki-assets`) -- Wiki viewing theme and asset bundle.
- `app` (`tomet-wiki`) -- Main CLI binary (`build`, `serve`, `init`).

## Theme & Reading Experience

`tomet-wiki-assets` provides an Askama-based theme tailored specifically for wikis and digital gardens:

- Single-Column Focused Reading: Eliminates book-style sidebar hierarchies and linear previous/next pagination in favor of organic graph navigation.
- First-Class Backlinks: Automatic presentation of incoming references at the footer of each document.
- Inline Conflict Visualization: Dedicated presentation for `.tm-conflict` blocks, clearly distinguishing local and remote versions with badge indicators.
- Editor & CMS Integration: Injects `data-tmt-rel-path`, `window.tmtWikiCms`, and the `#tmt-wiki-editor-root` container for Svelte editor components.

## Building and Testing

### Build workspace

```bash
just build
```

### Run tests

```bash
just test
```

### Check code

```bash
just check
just clippy
```

### Documentation

```bash
just docs        # regenerate Markdown from tmtroot/
just docs-check  # verify docs match exports
```

