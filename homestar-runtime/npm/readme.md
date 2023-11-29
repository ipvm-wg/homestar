# Homestar NPM packages

## Packages

- [homestar-runtime](https://www.npmjs.com/package/homestar-runtime) - This is the main package that installs the os specific binary package and runs it.
- [homestar-darwin-arm64](https://www.npmjs.com/package/homestar-darwin-arm64)
- [homestar-darwin-x64](https://www.npmjs.com/package/homestar-darwin-x64)
- [homestar-linux-arm64](https://www.npmjs.com/package/homestar-linux-arm64)
- [homestar-linux-x64](https://www.npmjs.com/package/homestar-linux-x64)
- [homestar-windows-x64](https://www.npmjs.com/package/homestar-windows-x64)

## Usage

```bash
npx homestar-runtime --help

# Global install
npm install -g homestar-runtime
homestar start -c config.toml
```

## Manual publishing

```bash

rustup target add aarch64-unknown-linux-gnu
rustup target add x86_64-unknown-linux-musl
cargo install cargo-get


export node_version=$(cargo get workspace.package.version)
export bin="homestar"


## darwin arm64
cargo build -p homestar-runtime --features ansi-logs --locked --release --target aarch64-apple-darwin
export node_os=darwin
export node_arch=arm64
export node_pkg="${bin}-${node_os}-${node_arch}"
mkdir -p "binaries/${node_pkg}/bin"
envsubst < package.json.tmpl > "binaries/${node_pkg}/package.json"
cp "../../target/aarch64-apple-darwin/release/${bin}" "binaries/${node_pkg}/bin"

## darwin x64
cross build -p homestar-runtime --features ansi-logs --locked --release --target x86_64-apple-darwin
export node_os=darwin
export node_arch=x64
export node_pkg="${bin}-${node_os}-${node_arch}"
mkdir -p "binaries/${node_pkg}/bin"
envsubst < package.json.tmpl > "binaries/${node_pkg}/package.json"
cp "../../target/x86_64-apple-darwin/release/${bin}" "binaries/${node_pkg}/bin"

## linux arm64
cross build -p homestar-runtime --features ansi-logs --locked --release --target aarch64-unknown-linux-gnu
export node_os=linux
export node_arch=arm64
export node_pkg="${bin}-${node_os}-${node_arch}"
mkdir -p "binaries/${node_pkg}/bin"
envsubst < package.json.tmpl > "binaries/${node_pkg}/package.json"
cp "../../target/aarch64-unknown-linux-gnu/release/${bin}" "binaries/${node_pkg}/bin"

## linux x64
cross build -p homestar-runtime --features ansi-logs --locked --release --target x86_64-unknown-linux-musl
export node_os=linux
export node_arch=x64
export node_pkg="${bin}-${node_os}-${node_arch}"
mkdir -p "binaries/${node_pkg}/bin"
envsubst < package.json.tmpl > "binaries/${node_pkg}/package.json"
cp "../../target/x86_64-unknown-linux-musl/release/${bin}" "binaries/${node_pkg}/bin"

# publish the RC package
cd "${node_pkg}"
npm version $(cargo get package.version)-rc.$(date +%s) --git-tag-version false
npm publish --access public --tag rc
```
