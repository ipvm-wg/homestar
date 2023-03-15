# syntax=docker/dockerfile:1

# AMD64
FROM --platform=$BUILDPLATFORM messense/rust-musl-cross:x86_64-musl as builder-amd64

# ARM64
FROM --platform=$BUILDPLATFORM messense/rust-musl-cross:aarch64-musl as builder-arm64

ARG TARGETARCH
FROM builder-$TARGETARCH as builder

RUN adduser --disabled-password --disabled-login --gecos "" --no-create-home homestar

RUN apt update && apt install -y protobuf-compiler sqlite

RUN cargo init

# touch lib.rs as we combine both
RUN touch src/lib.rs

# touch benches as it's part of Cargo.toml
RUN mkdir benches
RUN touch benches/a_benchmark.rs

# copy cargo.*
COPY Cargo.lock ./Cargo.lock
COPY Cargo.toml ./Cargo.toml

# cache depencies
RUN mkdir .cargo
RUN cargo vendor > .cargo/config
RUN --mount=type=cache,target=$CARGO_HOME/registry \
    --mount=type=cache,target=$CARGO_HOME/.git \
    --mount=type=cache,target=homestar-runtime/target,sharing=locked \
    cargo build --target $CARGO_BUILD_TARGET --release

# copy src
COPY src ./src
# copy benches
COPY benches ./benches

# final build for release
RUN rm ./target/$CARGO_BUILD_TARGET/release/deps/*homestar*
RUN --mount=type=cache,target=$CARGO_HOME/registry \
    --mount=type=cache,target=$CARGO_HOME/.git \
    --mount=type=cache,target=homestar/target,sharing=locked \
    cargo build --target $CARGO_BUILD_TARGET --bin homestar-runtime --release

RUN musl-strip ./target/$CARGO_BUILD_TARGET/release/homestar-runtime
RUN mv ./target/$CARGO_BUILD_TARGET/release/homestar-runtime /usr/local/bin

FROM scratch

ARG backtrace=0
ARG log_level=info

ENV RUST_BACKTRACE=${backtrace} \
    RUST_LOG=${log_level}

COPY --from=builder /usr/local/bin/homestar
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

USER homestar:homestar

ENTRYPOINT ["./homestar"]
