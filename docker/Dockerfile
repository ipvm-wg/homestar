# syntax=docker/dockerfile:1

# AMD64
FROM --platform=$BUILDPLATFORM messense/rust-musl-cross:x86_64-musl as builder-amd64

# ARM64
FROM --platform=$BUILDPLATFORM messense/rust-musl-cross:aarch64-musl as builder-arm64

ARG TARGETARCH
FROM builder-$TARGETARCH as builder

RUN adduser --disabled-password --disabled-login --gecos "" --no-create-home homestar

RUN cargo init
RUN rm -Rf src

RUN cargo init --lib homestar-core && \
    cargo init --lib homestar-functions && \
    cargo init --lib homestar-wasm && \
    cargo init --lib homestar-runtime

RUN echo "fn main() {}" > ./homestar-runtime/src/main.rs

# copy cargo.*
COPY Cargo.lock Cargo.toml .env diesel.toml ./
COPY ../homestar-core/Cargo.toml ./homestar-core/
COPY ../homestar-functions/Cargo.toml ./homestar-functions/
COPY ../homestar-wasm/Cargo.toml ./homestar-wasm/
COPY ../homestar-runtime/Cargo.toml ../homestar-runtime/migrations ./homestar-runtime/

# cache depencies
RUN mkdir .cargo
RUN cargo vendor > .cargo/config
RUN cargo install diesel_cli --no-default-features --features "sqlite-bundled"
RUN diesel setup
RUN diesel migration run
RUN --mount=type=cache,id=cargo,target=$CARGO_HOME/registry \
    --mount=type=cache,id=git,target=$CARGO_HOME/.git \
    --mount=type=cache,id=target,target=./target,sharing=locked \
    cargo build --workspace --target $CARGO_BUILD_TARGET --release

# copy workspace
COPY . ./

# final build for release
RUN cargo build --workspace --target $CARGO_BUILD_TARGET --release
RUN musl-strip ./target/$CARGO_BUILD_TARGET/release/homestar-runtime
RUN mv ./target/$CARGO_BUILD_TARGET/release/homestar-runtime /usr/local/bin/homestar-runtime

RUN mv ./*.db /etc/
RUN mv ./homestar-runtime/config /etc/config
RUN mv $CARGO_HOME/bin/diesel /usr/local/bin/diesel

FROM scratch

ARG backtrace=0
ARG log_level=info

ENV RUST_BACKTRACE=${backtrace} \
    RUST_LOG=${log_level}

COPY ../homestar-runtime/migrations .env diesel.toml ./
COPY --from=builder /usr/local/bin/homestar-runtime ./homestar
COPY --from=builder /usr/local/bin/diesel /usr/local/bin/diesel
COPY --from=builder /etc/*.db ./
COPY --from=builder /etc/config ./config
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

USER homestar:homestar

ENTRYPOINT ["./homestar"]
