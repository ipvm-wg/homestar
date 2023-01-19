{
  description = "ipvm";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs =
    { self
    , nixpkgs
    , flake-utils
    , rust-overlay
    } @ inputs:
    flake-utils.lib.eachDefaultSystem (system:
    let
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs { inherit system overlays; };

      rust-toolchain =
        (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml).override {
          extensions = [ "cargo" "clippy" "rustfmt" "rust-src" "rust-std" ];
        };

      nightly-rustfmt = pkgs.rust-bin.nightly.latest.rustfmt;

      format-pkgs = with pkgs; [
        nixpkgs-fmt
      ];

      macos = if pkgs.stdenv.isDarwin then [ pkgs.darwin.apple_sdk.frameworks.Foundation ] else [];

      cargo-installs = with pkgs; [
        cargo-deny
        cargo-expand
        cargo-outdated
        cargo-sort
        cargo-udeps
        cargo-watch
      ];
    in
    rec
    {
      devShells.default = pkgs.mkShell {
        name = "ipvm";
        nativeBuildInputs = with pkgs; [
          # The ordering of these two items is important. For nightly rustfmt to be used instead of
          # the rustfmt provided by `rust-toolchain`, it must appear first in the list. This is
          # because native build inputs are added to $PATH in the order they're listed here.
          nightly-rustfmt
          rust-toolchain
          rust-analyzer
          pre-commit
          protobuf
          sqlite
          diesel-cli
          direnv
          self.packages.${system}.irust
        ] ++ format-pkgs ++ cargo-installs ++ macos;

      shellHook = ''
        [ -e .git/hooks/pre-commit ] || pre-commit install --install-hooks && pre-commit install --hook-type commit-msg
      '';
      };

      packages.irust = pkgs.rustPlatform.buildRustPackage rec {
        pname = "irust";
        version = "1.66.0";
        src = pkgs.fetchFromGitHub {
          owner = "sigmaSd";
          repo = "IRust";
          rev = "v${version}";
          sha256 = "sha256-tqNTh8ojTT80kl9jDCMq289jSfPz82gQ37UFAmnfsOw=";
        };

        doCheck = false;
        cargoSha256 = "sha256-LXBQkiNU4vP6PIW+5iaEzonNGrpwnTMeVaosqLJoGCg=";
      };
    }
  );
}
