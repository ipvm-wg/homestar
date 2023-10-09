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
          targets = [
            "wasm32-unknown-unknown"
            "wasm32-wasi"
            "x86_64-apple-darwin"
            "aarch64-apple-darwin"
            "x86_64-unknown-linux-musl"
            "aarch64-unknown-linux-musl"
          ];
        };

        nightly-rustfmt = pkgs.rust-bin.nightly.latest.rustfmt;

        format-pkgs = with pkgs; [
          nixpkgs-fmt
          alejandra
          taplo
        ];

        cargo-installs = with pkgs; [
          cargo-deny
          cargo-deb
          cargo-cross
          cargo-expand
          cargo-nextest
          cargo-outdated
          cargo-sort
          cargo-spellcheck
          cargo-unused-features
          cargo-udeps
          cargo-watch
          rustup
          twiggy
          wasm-tools
        ];

        ci = pkgs.writeScriptBin "ci" ''
          #!${pkgs.stdenv.shell}
          cargo fmt --check
          cargo clippy
          cargo build --release
          nx-test
          nx-test-0
        '';

        db = pkgs.writeScriptBin "db" ''
          #!${pkgs.stdenv.shell}
          diesel setup
          diesel migration run
        '';

        dbReset = pkgs.writeScriptBin "db-reset" ''
          #!${pkgs.stdenv.shell}
          diesel database reset
          diesel setup
          diesel migration run
        '';

        devBuild = pkgs.writeScriptBin "cargo-build-dev" ''
          #!${pkgs.stdenv.shell}
          RUSTFLAGS="--cfg tokio_unstable" cargo build --no-default-features --features dev
        '';

        devCheck = pkgs.writeScriptBin "cargo-check-dev" ''
          #!${pkgs.stdenv.shell}
          RUSTFLAGS="--cfg tokio_unstable" cargo build --no-default-features --features dev
        '';

        devRunServer = pkgs.writeScriptBin "cargo-run-dev" ''
          #!${pkgs.stdenv.shell}
          cargo run --no-default-features --features dev -- start -c homestar-runtime/config/settings.toml
        '';

        doc = pkgs.writeScriptBin "doc" ''
          #!${pkgs.stdenv.shell}
          cargo doc --workspace --no-deps --document-private-items --open
        '';

        docAll = pkgs.writeScriptBin "doc-all" ''
          #!${pkgs.stdenv.shell}
          cargo doc --workspace --document-private-items --open
        '';

        dockerBuild = arch:
          pkgs.writeScriptBin "docker-${arch}" ''
            #!${pkgs.stdenv.shell}
            docker buildx build --file docker/Dockerfile --platform=linux/${arch} -t homestar-runtime --progress=plain .
          '';

        xFunc = cmd:
          pkgs.writeScriptBin "x-${cmd}" ''
            #!${pkgs.stdenv.shell}
            cargo watch -c -x ${cmd}
          '';

        xFuncAll = cmd:
          pkgs.writeScriptBin "x-${cmd}-all" ''
            #!${pkgs.stdenv.shell}
            cargo watch -c -s "cargo ${cmd} --workspace --all-features"
          '';

        xFuncNoDefault = cmd:
          pkgs.writeScriptBin "x-${cmd}-0" ''
            #!${pkgs.stdenv.shell}
            cargo watch -c -s "cargo ${cmd} --workspace --no-default-features"
          '';

        xFuncPackage = cmd: crate:
          pkgs.writeScriptBin "x-${cmd}-${crate}" ''
            #!${pkgs.stdenv.shell}
            cargo watch -c -s "cargo ${cmd} -p homestar-${crate} --all-features"
          '';

        xFuncTest = pkgs.writeScriptBin "x-test" ''
          #!${pkgs.stdenv.shell}
          cargo watch -c -s "cargo nextest run --workspace --nocapture && cargo test --doc"
        '';

        xFuncTestAll = pkgs.writeScriptBin "x-test-all" ''
          #!${pkgs.stdenv.shell}
          cargo watch -c -s "cargo nextest run --workspace --all-features --nocapture \
          && cargo test --workspace --doc --all-features"
        '';

        xFuncTestNoDefault = pkgs.writeScriptBin "x-test-0" ''
          #!${pkgs.stdenv.shell}
          cargo watch -c -s "cargo nextest run --workspace --no-default-features --nocapture \
          && cargo test --workspace --doc --no-default-features"
        '';

        xFuncTestPackage = crate:
          pkgs.writeScriptBin "x-test-${crate}" ''
            #!${pkgs.stdenv.shell}
            cargo watch -c -s "cargo nextest run -p homestar-${crate} --all-features \
            && cargo test --doc -p homestar-${crate} --all-features"
          '';

        nxTest = pkgs.writeScriptBin "nx-test" ''
          #!${pkgs.stdenv.shell}
          cargo nextest run --workspace
          cargo test --workspace --doc
        '';

        nxTestAll = pkgs.writeScriptBin "nx-test-all" ''
          #!${pkgs.stdenv.shell}
          cargo nextest run --workspace --all-features --nocapture
          cargo test --workspace --doc --all-features
        '';

        nxTestNoDefault = pkgs.writeScriptBin "nx-test-0" ''
          #!${pkgs.stdenv.shell}
          cargo nextest run --workspace --no-default-features --nocapture
          cargo test --workspace --doc --no-default-features
        '';

        wasmTest = pkgs.writeScriptBin "wasm-ex-test" ''
          #!${pkgs.stdenv.shell}
          cargo build -p homestar-functions-test --target wasm32-unknown-unknown --profile release-wasm-fn
          cp target/wasm32-unknown-unknown/release-wasm-fn/homestar_functions_test.wasm homestar-wasm/fixtures/example_test.wasm
          wasm-tools component new homestar-wasm/fixtures/example_test.wasm -o homestar-wasm/fixtures/example_test_component.wasm
        '';

        wasmAdd = pkgs.writeScriptBin "wasm-ex-add" ''
          #!${pkgs.stdenv.shell}
          cargo build -p homestar-functions-add --target wasm32-unknown-unknown --profile release-wasm-fn
          cp target/wasm32-unknown-unknown/release-wasm-fn/homestar_functions_add.wasm homestar-wasm/fixtures/example_add.wasm
          wasm-tools component new homestar-wasm/fixtures/example_add.wasm -o homestar-wasm/fixtures/example_add_component.wasm
          wasm-tools print homestar-wasm/fixtures/example_add.wasm -o homestar-wasm/fixtures/example_add.wat
          wasm-tools print homestar-wasm/fixtures/example_add_component.wasm -o homestar-wasm/fixtures/example_add_component.wat
        '';

        runIpfs = pkgs.writeScriptBin "run-ipfs" ''
          #!${pkgs.stdenv.shell}
          ipfs --repo-dir ./.ipfs --offline daemon
        '';

        scripts = [
          ci
          db
          dbReset
          devCheck
          devBuild
          devRunServer
          doc
          docAll
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
          runIpfs
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
              pkg-config
              pre-commit
              diesel-cli
              direnv
              nodejs_18
              kubo
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

          shellHook =
            ''
              [ -e .git/hooks/pre-commit ] || pre-commit install --install-hooks && pre-commit install --hook-type commit-msg

              # Setup local Kubo config
              if [ ! -e ./.ipfs ]; then
                ipfs --repo-dir ./.ipfs --offline init
              fi

              # Run Kubo / IPFS
              echo -e "To run Kubo as a local IPFS node, use the following command:"
              echo -e "ipfs --repo-dir ./.ipfs --offline daemon"
            ''
            # See https://github.com/nextest-rs/nextest/issues/267
            + (pkgs.lib.strings.optionalString pkgs.stdenv.isDarwin ''
              export DYLD_FALLBACK_LIBRARY_PATH="$(rustc --print sysroot)/lib"
            '');
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
