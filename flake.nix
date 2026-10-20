{
  description = "ptbr-writter dev shell — OpenCode + local Qwen tooling";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config = { allowUnfree = true; };
        };
      in
      {
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
