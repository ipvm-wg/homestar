{
  description = "homestar";

  inputs = {
    # we leverage unstable due to wasm-tools/wasm updates
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-compat,
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
          cargo-nextest
          cargo-outdated
          cargo-sort
          cargo-udeps
          cargo-watch
          twiggy
          wasm-tools
        ];

        ci = pkgs.writeScriptBin "ci" ''
          cargo fmt --check
          cargo clippy
          cargo build --release
          nx-test
          nx-test-0
        '';

        db = pkgs.writeScriptBin "db" ''
          diesel setup
          diesel migration run
        '';

        dbReset = pkgs.writeScriptBin "db-reset" ''
          diesel database reset
          diesel setup
          diesel migration run
        '';

        compileWasm = pkgs.writeScriptBin "compile-wasm" ''
          cargo build -p homestar-functions --target wasm32-unknown-unknown --release
        '';

        dockerBuild = arch:
          pkgs.writeScriptBin "docker-${arch}" ''
            docker buildx build --file docker/Dockerfile --platform=linux/${arch} -t homestar-runtime --progress=plain .
          '';

        xFunc = cmd:
          pkgs.writeScriptBin "x-${cmd}" ''
            cargo watch -c -x ${cmd}
          '';

        xFuncAll = cmd:
          pkgs.writeScriptBin "x-${cmd}-all" ''
            cargo watch -c -s "cargo ${cmd} --all-features"
          '';

        xFuncNoDefault = cmd:
          pkgs.writeScriptBin "x-${cmd}-0" ''
            cargo watch -c -s "cargo ${cmd} --no-default-features"
          '';

        xFuncPackage = cmd: crate:
          pkgs.writeScriptBin "x-${cmd}-${crate}" ''
            cargo watch -c -s "cargo ${cmd} -p homestar-${crate} --all-features"
          '';

        xFuncTest = pkgs.writeScriptBin "x-test" ''
          cargo watch -c -s "cargo nextest run --nocapture && cargo test --doc"
        '';

        xFuncTestAll = pkgs.writeScriptBin "x-test-all" ''
          cargo watch -c -s "cargo nextest run --all-features --nocapture \
          && cargo test --doc --all-features"
        '';

        xFuncTestNoDefault = pkgs.writeScriptBin "x-test-0" ''
          cargo watch -c -s "cargo nextest run --no-default-features --nocapture \
          && cargo test --doc --no-default-features"
        '';

        xFuncTestPackage = crate:
          pkgs.writeScriptBin "x-test-${crate}" ''
            cargo watch -c -s "cargo nextest run -p homestar-${crate} --all-features \
            && cargo test --doc -p homestar-${crate} --all-features"
          '';

        nxTest = pkgs.writeScriptBin "nx-test" ''
          cargo nextest run
          cargo test --doc
        '';

        nxTestAll = pkgs.writeScriptBin "nx-test-all" ''
          cargo nextest run --all-features --nocapture
          cargo test --doc --all-features
        '';

        nxTestNoDefault = pkgs.writeScriptBin "nx-test-0" ''
          cargo nextest run --no-default-features --nocapture
          cargo test --doc --no-default-features
        '';

        wasmTest = pkgs.writeScriptBin "wasm-ex-test" ''
          cargo build -p homestar-functions --features example-test --target wasm32-unknown-unknown --release
          cp target/wasm32-unknown-unknown/release/homestar_functions.wasm homestar-wasm/fixtures/example_test.wasm
          wasm-tools component new homestar-wasm/fixtures/example_test.wasm -o homestar-wasm/fixtures/example_test_component.wasm
        '';

        wasmAdd = pkgs.writeScriptBin "wasm-ex-add" ''
          cargo build -p homestar-functions --features example-add --target wasm32-unknown-unknown --release
          cp target/wasm32-unknown-unknown/release/homestar_functions.wasm homestar-wasm/fixtures/example_add.wasm
          wasm-tools component new homestar-wasm/fixtures/example_add.wasm -o homestar-wasm/fixtures/example_add_component.wasm
          wasm-tools print homestar-wasm/fixtures/example_add.wasm -o homestar-wasm/fixtures/example_add.wat
          wasm-tools print homestar-wasm/fixtures/example_add_component.wasm -o homestar-wasm/fixtures/example_add_component.wat
        '';

        scripts = [
          ci
          db
          dbReset
          compileWasm
          (builtins.map (arch: dockerBuild arch) ["amd64" "arm64"])
          (builtins.map (cmd: xFunc cmd) ["build" "check" "run" "clippy"])
          (builtins.map (cmd: xFuncAll cmd) ["build" "check" "run" "clippy"])
          (builtins.map (cmd: xFuncNoDefault cmd) ["build" "check" "run" "clippy"])
          (builtins.map (cmd: xFuncPackage cmd "core") ["build" "check" "run" "clippy"])
          (builtins.map (cmd: xFuncPackage cmd "wasm") ["build" "check" "run" "clippy"])
          (builtins.map (cmd: xFuncPackage cmd "runtime") ["build" "check" "run" "clippy"])
          xFuncTest
          xFuncTestAll
          xFuncTestNoDefault
          (builtins.map (crate: xFuncTestPackage crate) ["core" "wasm" "guest-wasm" "runtime"])
          nxTest
          nxTestAll
          nxTestNoDefault
          wasmTest
          wasmAdd
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
              diesel-cli
              direnv
              self.packages.${system}.irust
            ]
            ++ format-pkgs
            ++ cargo-installs
            ++ scripts
            ++ lib.optionals stdenv.isDarwin [
              darwin.apple_sdk.frameworks.Security
              darwin.apple_sdk.frameworks.CoreFoundation
              darwin.apple_sdk.frameworks.Foundation
            ];
          NIX_PATH = "nixpkgs=" + pkgs.path;
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

        formatter = pkgs.alejandra;
      }
    );
}
