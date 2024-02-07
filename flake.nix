{
  description = "homestar";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-23.11";
    # we leverage unstable due to wasm-tools/wasm updates
    nixos-unstable.url = "nixpkgs/nixos-unstable-small";
    flake-utils.url = "github:numtide/flake-utils";
    nixlib.url = "github:nix-community/nixpkgs.lib";

    nixify = {
      url = "github:rvolosatovs/nixify";
      inputs.nixlib.follows = "nixlib";
    };

    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    wit-deps = {
      url = "github:bytecodealliance/wit-deps/v0.3.5";
      inputs.nixlib.follows = "nixlib";
      inputs.nixify.follows = "nixify";
    };

    # Needed due to wit-deps not existing in nixpkgs.
    # TODO: Remove once wit-deps is in nixpkgs or move to fenix? maybe?
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    nixos-unstable,
    fenix,
    flake-compat,
    flake-utils,
    rust-overlay,
    wit-deps,
    ...
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay) fenix.overlays.default wit-deps.overlays.default];
        pkgs = import nixpkgs {inherit system overlays;};
        unstable = import nixos-unstable {inherit system overlays;};

        rust-toolchain = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml).override {
          extensions = ["cargo" "clippy" "llvm-tools-preview" "rustfmt" "rust-src" "rust-std"];
          targets = [
            "wasm32-unknown-unknown"
            "wasm32-wasi"
            "x86_64-apple-darwin"
            "aarch64-apple-darwin"
            "x86_64-unknown-linux-gnu"
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
          cargo-deb
          cargo-deny
          cargo-cross
          cargo-expand
          cargo-hakari
          cargo-machete
          cargo-nextest
          cargo-sort
          cargo-unused-features
          cargo-udeps
          cargo-watch
          rustup
          tokio-console
          twiggy
          unstable.cargo-component
          unstable.wasm-tools
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
          cargo run --no-default-features --features dev -- start
        '';

        doc = pkgs.writeScriptBin "doc" ''
          #!${pkgs.stdenv.shell}
          cargo doc --workspace --all-features --no-deps --document-private-items --open
        '';

        docAll = pkgs.writeScriptBin "doc-all" ''
          #!${pkgs.stdenv.shell}
          cargo doc --workspace --all-features --document-private-items --open
        '';

        dockerBuild = arch:
          pkgs.writeScriptBin "docker-${arch}" ''
            #!${pkgs.stdenv.shell}
            docker buildx build --build-arg git_sha=$(git rev-parse HEAD) \
            --build-arg git_timestamp=$(git log -1 --pretty=format:'%cI') \
            --file docker/Dockerfile --platform=linux/${arch} -t homestar --progress=plain .
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
            cargo watch -c -s "cargo ${cmd} --no-default-features"
          '';

        xFuncPackage = cmd: crate:
          pkgs.writeScriptBin "x-${cmd}-${crate}" ''
            #!${pkgs.stdenv.shell}
            cargo watch -c -s "cargo ${cmd} -p homestar-${crate} --all-features"
          '';

        xFuncTest = pkgs.writeScriptBin "x-test" ''
          #!${pkgs.stdenv.shell}
          cargo watch -c -s "cargo nextest run --workspace --no-capture && cargo test --doc"
        '';

        xFuncTestAll = pkgs.writeScriptBin "x-test-all" ''
          #!${pkgs.stdenv.shell}
          cargo watch -c -s "cargo nextest run --workspace --all-features --no-capture \
          && cargo test --workspace --doc --all-features"
        '';

        xFuncTestNoDefault = pkgs.writeScriptBin "x-test-0" ''
          #!${pkgs.stdenv.shell}
          cargo watch -c -s "cargo nextest run --no-default-features --no-capture \
          && cargo test --doc --no-default-features"
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
          cargo nextest run --workspace --all-features --no-capture
          cargo test --workspace --doc --all-features
        '';

        nxTestNoDefault = pkgs.writeScriptBin "nx-test-0" ''
          #!${pkgs.stdenv.shell}
          cargo nextest run --no-default-features --no-capture
          cargo test --doc --no-default-features
        '';

        wasmTest = pkgs.writeScriptBin "wasm-ex-test" ''
          #!${pkgs.stdenv.shell}
          cargo build -p homestar-functions-test --target wasm32-unknown-unknown --profile release-wasm-fn
          cp target/wasm32-unknown-unknown/release-wasm-fn/homestar_functions_test.wasm homestar-wasm/fixtures/example_test.wasm
          wasm-tools component new homestar-wasm/fixtures/example_test.wasm -o homestar-wasm/fixtures/example_test_component.wasm
          cargo build -p homestar-functions-test --target wasm32-wasi --profile release-wasm-fn
          cp target/wasm32-wasi/release-wasm-fn/homestar_functions_test.wasm homestar-wasm/fixtures/example_test_wasi.wasm
          wasm-tools component new homestar-wasm/fixtures/example_test_wasi.wasm -o homestar-wasm/fixtures/example_test_wasi_component.wasm --adapt homestar-functions/wasi_snapshot_preview1.wasm
          cp homestar-wasm/fixtures/example_test.wasm examples/websocket-relay/example_test.wasm
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

        testCleanup = pkgs.writeScriptBin "test-clean" ''
          #!${pkgs.stdenv.shell}
          rm -rf homestar-runtime/tests/fixtures/*.db*
          rm -rf homestar-runtime/tests/fixtures/*.toml
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
          testCleanup
          wasmTest
          wasmAdd
        ];
      in {
        devShells.default = pkgs.mkShell {
          name = "homestar";
          nativeBuildInputs = with pkgs;
            [
              # The ordering of these two items is important. For nightly rustfmt to be used instead of
              # the rustfmt provided by `rust-toolchain`, it must appear first in the list. This is
              # because native build inputs are added to $PATH in the order they're listed here.
              nightly-rustfmt
              rust-toolchain
              pkgs.wit-deps
              pkg-config
              pre-commit
              diesel-cli
              direnv
              unstable.nodejs_20
              unstable.nodePackages.pnpm
              action-validator
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

              unset SOURCE_DATE_EPOCH

              # Run Kubo / IPFS
              echo -e "To run Kubo as a local IPFS node, use the following command:"
              echo -e "ipfs --repo-dir ./.ipfs --offline daemon"
            ''
            # See https://github.com/nextest-rs/nextest/issues/267
            + (pkgs.lib.strings.optionalString pkgs.stdenv.isDarwin ''
              export DYLD_FALLBACK_LIBRARY_PATH="$(rustc --print sysroot)/lib"
              export NIX_LDFLAGS="-F${pkgs.darwin.apple_sdk.frameworks.CoreFoundation}/Library/Frameworks -framework CoreFoundation $NIX_LDFLAGS";
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

        packages.default =
          pkgs.rustPlatform.buildRustPackage
          {
            name = "homestar";
            src = ./.;
            cargoLock = {
              lockFile = ./Cargo.lock;
            };
            buildInputs = with pkgs;
              [rust-toolchain]
              ++ lib.optionals stdenv.isDarwin [
                darwin.apple_sdk.frameworks.Security
                darwin.apple_sdk.frameworks.CoreFoundation
                darwin.apple_sdk.frameworks.Foundation
              ];

            doCheck = false;
          };

        formatter = pkgs.alejandra;
      }
    );
}
