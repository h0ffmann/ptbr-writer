{
  description = "ptbr-writter dev shell — OpenCode + local Qwen tooling";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    # The site's toolchain (Gleam on JavaScript, Lustre, node, esbuild, just), as the sister sites
    # h0ffmann/closed-source and h0ffmann/ww3-gpu take it. flake.lock pins it to the revision of
    # the forks/picasso submodule; `just pins` fails when the two differ.
    gleam-website.url = "github:h0ffmann/picasso?dir=labs/gleam-website";
  };

  outputs = { self, nixpkgs, flake-utils, gleam-website }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config = { allowUnfree = true; };
        };
      in
      {
        devShells.site = gleam-website.devShells.${system}.default;
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            just
            python3
            git
            jq
            curl
            lsof
            iproute2
          ];

          shellHook = ''
            echo "ptbr-writter dev shell"
            echo "  just opencode-qwen   serve Qwen3.6-27B and run OpenCode"
            echo "  just opencode        run OpenCode against the configured server"
            echo "  just quality         run basic checks"
          '';
        };
      });
}
