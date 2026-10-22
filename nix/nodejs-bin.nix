# Copied from h0ffmann/picasso labs/gleam-website/nodejs-bin.nix (h0ffmann/picasso#143): a lab references
# nothing outside itself, so this repository keeps its own copy. Same version, same hashes.
# Node.js 24.22.1, the official build from nodejs.org, in place of nixpkgs' nodejs until nixpkgs
# ships it: 24.22.1 is the October 2026 security release (Brotli RCE, TLS hostname bypass,
# nodejs.org/en/blog/vulnerability/october-2026-security-releases), and nixos-unstable is still on
# 24.21.0 (2026-10-22). The hashes are the release's SHASUMS256.txt. Drop this file, and the
# overlay in flake.nix, once `nix flake update` brings nodejs >= 24.22.1.
{ lib, stdenv, fetchurl, patchelf }:

let
  version = "24.22.1";
  builds = {
    x86_64-linux = { arch = "linux-x64"; sha256 = "6c172a9ab6718f7b801fb5a7bd486af2ee5e32758086a04b84fcce2aa77095e6"; };
    aarch64-linux = { arch = "linux-arm64"; sha256 = "dbabf7a1cbf947e6059ffad2c27b80041db97dea1e553b62afdee7f13fca336e"; };
    aarch64-darwin = { arch = "darwin-arm64"; sha256 = "654ba79de6155647e22321932b45f2a1ccb705976cc337c9b8e278beb085a628"; };
    x86_64-darwin = { arch = "darwin-x64"; sha256 = "65e450f90a6eaacd5000f611861c4ce65fadc87f813795dc4ed0c0e5f2fca2d4"; };
  };
  build = builds.${stdenv.hostPlatform.system} or (throw "nodejs-bin: no official build for ${stdenv.hostPlatform.system}");
in
stdenv.mkDerivation {
  pname = "nodejs";
  inherit version;
  src = fetchurl {
    url = "https://nodejs.org/dist/v${version}/node-v${version}-${build.arch}.tar.xz";
    inherit (build) sha256;
  };
  nativeBuildInputs = lib.optionals stdenv.hostPlatform.isLinux [ patchelf ];
  dontConfigure = true;
  dontBuild = true;
  # The fixup phase must not touch the ELF again: strip and shrink-rpath both break node.
  dontStrip = true;
  dontPatchELF = true;
  installPhase = ''
    runHook preInstall
    mkdir -p $out
    cp -r bin include lib share $out/
  '' + lib.optionalString stdenv.hostPlatform.isLinux ''
    # The rpath first, then the interpreter, in two calls: patchelf 0.19.2 makes a node that
    # segfaults in the loader when it sets both in one call, or the interpreter first.
    patchelf --set-rpath ${lib.makeLibraryPath [ stdenv.cc.cc.lib ]} $out/bin/node
    patchelf --set-interpreter "$(cat $NIX_CC/nix-support/dynamic-linker)" $out/bin/node
  '' + ''
    runHook postInstall
  '';
  # A node that does not start fails the build here, not in a user's shell.
  doInstallCheck = true;
  installCheckPhase = ''
    $out/bin/node --version | grep -qx "v${version}"
  '';
  meta = {
    description = "Node.js ${version}, the official nodejs.org build";
    homepage = "https://nodejs.org";
    license = lib.licenses.mit;
    mainProgram = "node";
    platforms = builtins.attrNames builds;
  };
}
