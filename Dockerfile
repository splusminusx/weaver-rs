FROM debian:stretch-slim as build

ENV RUSTUP_HOME=/rust
ENV CARGO_HOME=/cargo
ENV PATH=/cargo/bin:/rust/bin:$PATH

RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        curl \
        git \
        ssh \
        libssl-dev \
        cmake \
        pkg-config && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

RUN (curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly --no-modify-path) && rustup default nightly

WORKDIR /weaver

COPY . .

RUN cargo build --release

# Create a new stage with a minimal image
# because we already have a binary built
FROM debian:stretch-slim

RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends libssl1.1 && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Copies the binary from the "build" stage to the current stage
COPY --from=build weaver/target/release/server ./bin/
COPY --from=build weaver/target/release/client ./bin/
COPY --from=build weaver/target/release/producer ./bin/
COPY --from=build weaver/etc/config.toml ./etc/

EXPOSE 5000
EXPOSE 8085

ENTRYPOINT ["./bin/server", "--config", "./etc/config.toml"]
