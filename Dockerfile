# Multi-stage Dockerfile: build UI + Rust binary, then minimal runtime

# STAGE 1: build API Rust binary and UI
FROM rust:1.86 AS rust-builder

WORKDIR /mettakg

# Install node for UI build
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get update && apt-get install -y --no-install-recommends \
    nodejs musl-tools g++ libpq-dev libssl-dev pkg-config && \
    rustup target add x86_64-unknown-linux-musl

COPY api api
COPY frontend frontend

ENV OPENSSL_STATIC=1 \
    OPENSSL_DIR=/usr \
    OPENSSL_INCLUDE_DIR=/usr/include \
    OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu \
    LIBPQ_STATIC=1

# Build release binary (build.rs will build UI and embed it)
RUN cd api/ && cargo build --release --target x86_64-unknown-linux-musl

# STAGE 2: build and install translation sub-project (dev-only dependency)
FROM python:3.11-alpine AS python-builder

WORKDIR /mettakg

RUN apk add --no-cache \
    gcc g++ musl-dev \
    libffi-dev openssl-dev \
    python3-dev

COPY translations /mettakg/translations

RUN python3 -m venv /mettakg/venv && \
    /mettakg/venv/bin/pip install --no-deps --no-cache-dir -r /mettakg/translations/requirements.txt && \
    /mettakg/venv/bin/pip uninstall -y pip setuptools wheel

# STAGE 3: runtime image only ships the binary and translations venv
FROM python:3.11.7-alpine3.19

WORKDIR /mettakg

COPY --from=rust-builder /mettakg/api/target/x86_64-unknown-linux-musl/release/metta-kg /usr/local/bin/
COPY --from=python-builder /mettakg/venv /mettakg/venv
COPY --from=python-builder /mettakg/translations /mettakg/translations

# Rocket.toml not strictly needed; we use clap flags and env
RUN mkdir -p temp

ENTRYPOINT ["metta-kg"]
