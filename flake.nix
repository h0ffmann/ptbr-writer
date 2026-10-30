{
  description = "ptbr-writer dev shell — OpenCode + local Qwen tooling";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    # The site's toolchain (Gleam on JavaScript, Lustre, node, esbuild, just), as the sister sites
    # h0ffmann/closed-source and h0ffmann/ww3-gpu take it. flake.lock pins it to the revision of
    # the forks/picasso submodule; `just pins` fails when the two differ.
    gleam-website.url = "github:h0ffmann/picasso?dir=labs/gleam-website";
    # The code graph of this repository (site/dist/grafo/graph.html), from picasso's code-graph lab
    # at the same revision (`just pins`).
    code-graph.url = "github:h0ffmann/picasso?dir=labs/code-graph";
  };

  outputs = { self, nixpkgs, flake-utils, gleam-website, code-graph }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config = { allowUnfree = true; };
        };
        # The icon (drawings/, pure Nix): a G in the colours of the flag of Brazil.
        icon = import ./drawings { inherit (nixpkgs) lib; };
      in
      {
        # The icon: icon.svg, icon.desmos.json and icon-check.svg (`just icon` copies the first two).
        packages.icon-art = pkgs.linkFarm "icon-art" [
          { name = "icon.svg"; path = pkgs.writeText "icon.svg" icon.svg; }
          { name = "icon.desmos.json"; path = pkgs.writeText "icon.desmos.json" icon.desmos; }
          { name = "icon-check.svg"; path = pkgs.writeText "icon-check.svg" icon.checkSvg; }
        ];

        # The icon's tests pass and the committed files are its output (`just icon`).
        checks.icon = pkgs.runCommand "icon-art-check" { } ''
          ${pkgs.lib.concatMapStrings (f: "echo 'failed: ${f.name}' >&2\n") icon.failures}
          ${pkgs.lib.optionalString (icon.failures != [ ]) "exit 1"}
          art=${self.packages.${system}.icon-art}
          diff -u ${./site/static/favicon.svg} $art/icon.svg || { echo 'run `just icon`' >&2; exit 1; }
          diff -u ${./app/src-tauri/icons/icon.svg} $art/icon.svg || { echo 'run `just icon`' >&2; exit 1; }
          diff -u ${./docs/drawings/icon.desmos.json} $art/icon.desmos.json || { echo 'run `just icon`' >&2; exit 1; }
          touch $out
        '';

        devShells.site = pkgs.mkShell {
          inputsFrom = [ gleam-website.devShells.${system}.default ];
          packages = [ code-graph.packages.${system}.default pkgs.jq ];
        };
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            just
            opencode # `just opencode` and `just opencode-qwen`
            git
            jq
            curl
            lsof
            vale # the Saramago prose linter (lint/saramago)
            # node 24.22.1, the October 2026 security release (#83), from nodejs.org until nixpkgs
            # ships it: runs vendor/archify (`just figure`) and the extension's build and tests.
            # `just node-check` fails below it. Chrome comes from the host.
            (pkgs.callPackage ./nix/nodejs-bin.nix { })
            librsvg # rsvg-convert: the whitepaper's logos, SVG to PDF (`just whitepaper`)
            poppler-utils # pdfinfo: the whitepaper's 5-page cap; pandoc and XeLaTeX come from the host
            iproute2
            rustc # the extension (extension/Cargo.toml), Rust to wasm32-unknown-unknown
            cargo # `cargo test`, `cargo xtask build` (extension)
            clippy # `cargo clippy` (extension)
            rustfmt # `cargo fmt` (extension)
            lld # the wasm linker rustc calls
            wasm-bindgen-cli # the JS glue of the extension's wasm; its version is pinned in extension/Cargo.toml
            binaryen # wasm-opt: shrinks the extension's wasm (`cargo xtask build`)
          ];

          shellHook = ''
            echo "ptbr-writer dev shell"
            echo "  just opencode-qwen   serve Qwen3.6-27B and run OpenCode"
            echo "  just opencode        run OpenCode against the configured server"
            echo "  just quality         run basic checks"
          '';
        };
      });
}
