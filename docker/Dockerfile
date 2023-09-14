# syntax=docker/dockerfile:1

# AMD64
FROM --platform=$BUILDPLATFORM messense/rust-musl-cross:x86_64-musl as builder-amd64

# ARM64
FROM --platform=$BUILDPLATFORM messense/rust-musl-cross:aarch64-musl as builder-arm64

ARG TARGETARCH
FROM builder-$TARGETARCH as builder

ARG database_url="homestar.db"
ENV DATABASE_URL=${database_url}

RUN adduser --disabled-password --disabled-login --gecos "" --no-create-home homestar

RUN cargo init
RUN rm -Rf src

RUN cargo init --lib homestar-core && \
    cargo init --lib homestar-wasm && \
    cargo init --lib homestar-runtime

RUN echo "fn main() {}" > ./homestar-runtime/src/main.rs

RUN mkdir -p ./homestar-runtime/src/test_utils/proc_macro ./homestar-runtime/migrations ./examples ./homestar-functions
RUN bash -c 'pushd ./examples && cargo init dummy-app-examples && popd'
RUN bash -c 'pushd ./homestar-functions && cargo init dummy-app-fns && popd'

# copy cargo.*
COPY Cargo.lock Cargo.toml diesel.toml ./
COPY ../homestar-core/Cargo.toml ./homestar-core/
COPY ../homestar-wasm/Cargo.toml ./homestar-wasm/
COPY ../homestar-runtime/Cargo.toml ./homestar-runtime/
COPY ../homestar-runtime/migrations ./homestar-runtime/migrations
COPY ../homestar-runtime/src/test_utils/proc_macro ./homestar-runtime/src/test_utils/proc_macro

ENTRYPOINT ["/bin/bash"]

# cache depencies
RUN mkdir .cargo
RUN cargo vendor > .cargo/config
RUN --mount=type=cache,target=$CARGO_HOME/registry \
    cargo install diesel_cli --no-default-features --features "sqlite-bundled"
RUN diesel setup --database-url $DATABASE_URL
RUN diesel migration run --migration-dir ./homestar-runtime/migrations
RUN --mount=type=cache,id=cargo,target=$CARGO_HOME/registry \
    --mount=type=cache,id=git,target=$CARGO_HOME/.git \
    --mount=type=cache,id=target,target=./target,sharing=locked \
    cargo build --target $CARGO_BUILD_TARGET --release

# copy default-members workspace
COPY ../homestar-core ./homestar-core
COPY ../homestar-wasm ./homestar-wasm
COPY ../homestar-runtime ./homestar-runtime

# final build for release
RUN cargo build -p homestar-runtime --target $CARGO_BUILD_TARGET --release --offline

RUN mv ./*.db /etc/
RUN musl-strip ./target/$CARGO_BUILD_TARGET/release/homestar-runtime
RUN mv ./target/$CARGO_BUILD_TARGET/release/homestar-runtime /usr/local/bin/homestar-runtime
RUN mv ./homestar-runtime/config /etc/config
RUN mv $CARGO_HOME/bin/diesel /usr/local/bin/diesel
RUN chmod +x /usr/local/bin/diesel

FROM scratch

ARG backtrace=0
ARG log_levels=homestar_runtime=info,libp2p=info,tarpc=info

ENV RUST_BACKTRACE=${backtrace} \
    RUST_LOG=${log_levels}

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

USER homestar:homestar
ENV HOME=/home/runner USER=homestar GROUP=homestar
WORKDIR /home/runner

COPY --chown=homestar:homestar diesel.toml ./
COPY --chown=homestar:homestar ../homestar-runtime/migrations ./migrations
COPY --chown=homestar:homestar --from=builder /usr/local/bin/homestar-runtime ./homestar
COPY --chown=homestar:homestar --from=builder /usr/local/bin/diesel /usr/local/bin/diesel
COPY --chown=homestar:homestar --from=builder  /etc/*.db ./
COPY --chown=homestar:homestar --from=builder /etc/config ./config

ARG database_url="homestar.db"
ARG rpc_host="127.0.0.1"
ARG rpc_port=3030
ARG ws_port=1337

ENV DATABASE_URL=${database_url} \
    HOMESTAR__NODE__NETWORK__RPC_HOST=${rpc_host} \
    HOMESTAR__NODE__NETWORK__RPC_PORT=${rpc_port} \
    HOMESTAR__NODE__NETWORK__WS_PORT=${ws_port}

EXPOSE ${rpc_port} ${ws_port}

ENTRYPOINT ["./homestar",  "start", "--db", "homestar.db"]
