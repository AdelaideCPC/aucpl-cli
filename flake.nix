{
  description = "AUCPL CLI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    ...
  }: let
    systems = [
      "aarch64-darwin"
      "aarch64-linux"
      "x86_64-linux"
      "x86_64-pc-windows-gnu"
      "x86_64-pc-windows-msvc"
    ];

    overlays = {
      rust-overlay = rust-overlay.overlays.default;
      rust-toolchain = final: prev: {
        rustToolchain = final.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      };
    };

    mkPkgs = system:
      import nixpkgs {
        inherit system;
        overlays = builtins.attrValues overlays;
      };

    forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f (mkPkgs system));
  in {
    packages = forAllSystems (pkgs: {
      # Add packages as necessary
    });

    devShells = forAllSystems (pkgs: {
      default = import ./nix/shell.nix {inherit pkgs;};
    });

    formatter = forAllSystems (pkgs: pkgs.alejandra);

    overlays =
      overlays
      // {
        default = nixpkgs.lib.composeManyExtensions (builtins.attrValues overlays);
      };
  };
}
