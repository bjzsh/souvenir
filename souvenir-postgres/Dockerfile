# The official Postgres image with the `souvenir` extension installed and
# created in the default database and in template1 (so every new database
# has it).
#
# Build from the repository root (the extension uses the `souvenir` crate):
#
#   docker build -f souvenir-postgres/Dockerfile --build-arg PG_MAJOR=18 \
#     -t souvenir-postgres:18 .

ARG PG_MAJOR=18

# Build against the same image, so the extension matches its Postgres and C
# library exactly. The image already has the PGDG apt repository, which
# provides the server headers.
FROM postgres:${PG_MAJOR} AS build
ARG PG_MAJOR
ARG RUST_VERSION=1.97.0
ARG PGRX_VERSION=0.19.4

RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      build-essential ca-certificates clang curl libclang-dev pkg-config \
      "postgresql-server-dev-${PG_MAJOR}" \
 && rm -rf /var/lib/apt/lists/*

ENV CARGO_HOME=/usr/local/cargo RUSTUP_HOME=/usr/local/rustup PATH=/usr/local/cargo/bin:$PATH
RUN curl -fsSL https://sh.rustup.rs \
      | sh -s -- -y --profile minimal --default-toolchain "${RUST_VERSION}" \
 && cargo install cargo-pgrx --version "${PGRX_VERSION}" --locked \
 && cargo pgrx init "--pg${PG_MAJOR}=/usr/lib/postgresql/${PG_MAJOR}/bin/pg_config"

WORKDIR /src
COPY . .
RUN cd souvenir-postgres \
 && cargo pgrx package \
      --pg-config "/usr/lib/postgresql/${PG_MAJOR}/bin/pg_config" \
      --no-default-features --features "pg${PG_MAJOR}" \
      --out-dir /out

FROM postgres:${PG_MAJOR}
LABEL org.opencontainers.image.title="souvenir-postgres" \
      org.opencontainers.image.description="Postgres with the souvenir extension: a native type for Souvenir IDs" \
      org.opencontainers.image.source="https://github.com/bjzsh/souvenir" \
      org.opencontainers.image.licenses="MIT"
COPY --from=build /out/ /
COPY souvenir-postgres/docker/initdb.sql /docker-entrypoint-initdb.d/10-souvenir.sql
