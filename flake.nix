{
  description = "Flake for the cfg-fuzzer program";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs =
    { nixpkgs, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forEachSystem = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      formatter = forEachSystem (pkgs: pkgs.nixfmt);

      devShells = forEachSystem (pkgs: {
        default = pkgs.callPackage (
          {
            mkShell,
            cargo,
            rustc,
          }:
          mkShell {
            strictDeps = true;
            packages = [
              cargo
              rustc
            ];
          }
        ) { mkShell = pkgs.mkShell.override { inherit (pkgs.llvmPackages) stdenv; }; };
      });
    };
}
