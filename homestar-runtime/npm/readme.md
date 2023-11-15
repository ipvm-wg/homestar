# Homestar NPM packages

## Usage

```bash

rustup target add aarch64-unknown-linux-musl
rustup target add x86_64-unknown-linux-musl


export node_version=0.0.2
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
cross build -p homestar-runtime --features ansi-logs --locked --release --target aarch64-unknown-linux-musl
export node_os=linux
export node_arch=arm64
export node_pkg="${bin}-${node_os}-${node_arch}"
mkdir -p "binaries/${node_pkg}/bin"
envsubst < package.json.tmpl > "binaries/${node_pkg}/package.json"
cp "../../target/aarch64-unknown-linux-musl/release/${bin}" "binaries/${node_pkg}/bin"

## linux x64
cross build -p homestar-runtime --features ansi-logs --locked --release --target x86_64-unknown-linux-musl
export node_os=linux
export node_arch=x64
export node_pkg="${bin}-${node_os}-${node_arch}"
mkdir -p "binaries/${node_pkg}/bin"
envsubst < package.json.tmpl > "binaries/${node_pkg}/package.json"
cp "../../target/x86_64-unknown-linux-musl/release/${bin}" "binaries/${node_pkg}/bin"

# publish the package
cd "${node_pkg}"
npm publish --access public
```

## TODO

- [ ] move this to CI
- [ ] add windows
