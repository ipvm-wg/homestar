{
  description = "homestar";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-22.11";
    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {inherit system overlays;};

        rust-toolchain = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml).override {
          extensions = ["cargo" "clippy" "rustfmt" "rust-src" "rust-std"];
          targets = ["wasm32-unknown-unknown" "wasm32-wasi"];
        };

        nightly-rustfmt = pkgs.rust-bin.nightly.latest.rustfmt;

        format-pkgs = with pkgs; [
          nixpkgs-fmt
          alejandra
        ];

        cargo-installs = with pkgs; [
          cargo-deny
          cargo-expand
          cargo-outdated
          cargo-sort
          cargo-udeps
          cargo-watch
        ];
      in rec
      {
        devShells.default = pkgs.mkShell {
          name = "homestar";
          nativeBuildInputs = with pkgs;
            [
              # The ordering of these two items is important. For nightly rustfmt to be used instead of
              # the rustfmt provided by `rust-toolchain`, it must appear first in the list. This is
              # because native build inputs are added to $PATH in the order they're listed here.
              nightly-rustfmt
              rust-toolchain
              rust-analyzer
              pkg-config
              pre-commit
              protobuf
              rustup
              diesel-cli
              direnv
              self.packages.${system}.irust
            ]
            ++ format-pkgs
            ++ cargo-installs
            ++ lib.optionals stdenv.isDarwin [
              darwin.apple_sdk.frameworks.Security
              darwin.apple_sdk.frameworks.CoreFoundation
              darwin.apple_sdk.frameworks.Foundation
            ];
          RUST_BACKTRACE = 1;

          shellHook = ''
            [ -e .git/hooks/pre-commit ] || pre-commit install --install-hooks && pre-commit install --hook-type commit-msg
          '';
        };

        packages.irust = pkgs.rustPlatform.buildRustPackage rec {
          pname = "irust";
          version = "1.70.0";
          src = pkgs.fetchFromGitHub {
            owner = "sigmaSd";
            repo = "IRust";
            rev = "v${version}";
            sha256 = "sha256-chZKesbmvGHXwhnJRZbXyX7B8OwJL9dJh0O1Axz/n2E=";
          };

          doCheck = false;
          cargoSha256 = "sha256-FmsD3ajMqpPrTkXCX2anC+cmm0a2xuP+3FHqzj56Ma4=";
        };
      }
    );
}
