# tomet-wiki development tasks.

default:
    @just --list

#[ Build ]

build:
    cargo build --workspace

release:
    cargo build --workspace --release

# Build the WebAssembly of the editor (crates/tomet-wiki-wasm/pkg)
wasm:
    cargo build -p tomet-wiki-wasm --target wasm32-unknown-unknown --profile wasm
    wasm-bindgen --target web --out-dir crates/tomet-wiki-wasm/pkg target/wasm32-unknown-unknown/wasm/tomet_wiki_wasm.wasm

# Compile Svelte 5 Wiki editor bundle
editor: wasm
    deno run --min-dep-age=0 --node-modules-dir=auto -A crates/tomet-wiki-assets/build-editor.ts

# Verify editor bundle is up to date with sources
editor-check: editor
    git diff --exit-code crates/tomet-wiki-assets/assets/wiki-editor.js

#[ Test & lint ]

test:
    cargo test --workspace

check:
    cargo check --workspace --all-targets

clippy:
    cargo clippy --workspace --all-targets -- -D warnings

fmt:
    nix fmt

#[ Housekeeping ]

clean:
    cargo clean
    rm -rf dist result

#[ Documentation ]

# Generate and update all derived documentation across the workspace.
docs:
    tomet export .
    tomet format -i .

# Check that all documents parse, format cleanly, and match export targets.
docs-check:
    tomet check .
    tomet format --check .
    tomet export --check .
    twrit check .
    catasto check .
