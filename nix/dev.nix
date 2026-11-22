{
  pkgs,
  mkShell,
  fenix,
  tomet,
  twrit,
  catasto,
  sisho,
  ...
}:
let
  rustToolchain = fenix.combine [
    (fenix.stable.withComponents [
      "cargo"
      "clippy"
      "rustc"
      "rust-src"
      "rustfmt"
      "rust-analyzer"
    ])
    fenix.targets.wasm32-unknown-unknown.stable.rust-std
  ];
in
mkShell {
  buildInputs = with pkgs; [
    tomet
    twrit
    catasto
    sisho
    just
    pagefind
    deno

    #= Build
    pkg-config

    #= Rust
    rustToolchain
    cargo-edit
    cargo-outdated
    cargo-nextest
    wasm-bindgen-cli
  ];

  shellHook = ''
    echo "🦀📚 Rust Tomet (tomet-wiki)"
  '';
}
