{pkgs ? import <nixpkgs> {}}: let
  inherit (pkgs) lib;
  aucpl-dev = pkgs.writeShellScriptBin "aucpl" ''
    if [ -z "''${AUCPL_DEV_WORKSPACE_ROOT:-}" ]; then
      echo "AUCPL_DEV_WORKSPACE_ROOT is not set" >&2
      exit 1
    fi

    exec cargo run --quiet --manifest-path "''${AUCPL_DEV_WORKSPACE_ROOT}/Cargo.toml" --bin aucpl -- "$@"
  '';
in
  pkgs.mkShell rec {
    packages =
      [
        aucpl-dev
        pkgs.rust-analyzer
      ]
      ++ lib.optionals pkgs.stdenv.hostPlatform.isLinux [
        pkgs.mold
      ];

    buildInputs = [
      pkgs.rustToolchain
    ];

    LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;

    shellHook = ''
      export AUCPL_DEV_WORKSPACE_ROOT="$(dirname "$(cargo locate-project --workspace --message-format plain)")"
      eval "$(command aucpl shellinit)"
    '';
  }
