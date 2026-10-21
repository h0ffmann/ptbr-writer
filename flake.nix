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
      in
      {
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
            nodejs # runs vendor/archify (`just figure`); Chrome comes from the host
            iproute2
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
