<!-- Generated from tmtroot/readme.tmt. Edit that, then `tomet export .`. -->

# Tomet Wiki

Wiki system for the Tomet ecosystem.

## Overview

`tomet-wiki` is a Wiki engine designed for Tomet documents.

## Usage

```bash
tomet-wiki [path]              # open wiki for path (defaults to ".")
```

## Workspace Layout

- `app` (`tomet-wiki`) -- Main CLI binary.

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

