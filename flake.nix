{
  description = "signal-system - Signal contract for system → router focus observations";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-build = {
      url = "github:LiGoldragon/rust-build";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-build }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        rust = rust-build.lib.${system}.fromPkgs pkgs;
        inherit (rust) craneLib toolchain;
        examplesFilter = path: _type: builtins.match ".*/examples(/.*)?$" path != null;
        contractFilter = path: type:
          type == "regular" && (
            pkgs.lib.hasSuffix ".ethos" path ||
            pkgs.lib.hasSuffix "/build.rs" path ||
            builtins.match ".*/src/generated(/.*)?$" path != null
          );
        src = rust.cleanSource { root = ./.; extraFilters = [ examplesFilter contractFilter ]; };
        cargoVendorDirectory = craneLib.vendorCargoDeps { inherit src; };
        commonArguments = {
          inherit src cargoVendorDirectory;
          strictDeps = true;
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArguments;
      in
      {
        packages.default = craneLib.buildPackage (commonArguments // { inherit cargoArtifacts; });
        checks = {
          build = craneLib.cargoBuild (commonArguments // { inherit cargoArtifacts; });
          test = craneLib.cargoTest (commonArguments // { inherit cargoArtifacts; });
          test-datom = craneLib.cargoTest (commonArguments // { inherit cargoArtifacts; cargoTestExtraArgs = "--features datom"; });
          doc = craneLib.cargoDoc (commonArguments // {
            inherit cargoArtifacts;
            RUSTDOCFLAGS = "-D warnings";
          });
          fmt = craneLib.cargoFmt { inherit src; };
          clippy = craneLib.cargoClippy (commonArguments // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets --all-features -- -D warnings";
          });
          no-free-functions = pkgs.runCommand "signal-system-no-free-functions" { inherit src; } ''
            ${builtins.readFile ./checks/no-free-functions.sh}
          '';
          no-inherent-methods = pkgs.runCommand "signal-system-no-inherent-methods" { inherit src; } ''
            ${builtins.readFile ./checks/no-inherent-methods.sh}
          '';
        };
        devShells.default = pkgs.mkShell {
          name = "signal-system";
          packages = [ pkgs.jujutsu pkgs.pkg-config toolchain ];
        };
      });
}
