FROM node:22-trixie AS web
WORKDIR /src/web
COPY web/package.json web/package-lock.json ./
RUN npm ci
COPY web/ ./
RUN npm run check && npm run lint && npm run build

FROM rust:trixie AS rust
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
COPY migrations/ ./migrations/
COPY docs/openapi.json ./docs/openapi.json
COPY provider-engine/registry.json ./provider-engine/registry.json
RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/src/target cargo build --release --locked -p ddnsupd-api && cp target/release/ddnsupd-api /ddnsupd

FROM gcr.io/distroless/cc-debian13:nonroot@sha256:08476c244888bfb425bae1ae4dd7c17f093e972aab8426883d97f3c6f280f7c1
WORKDIR /app
COPY --from=rust /ddnsupd /app/ddnsupd
COPY --from=web /src/web/build /app/web
COPY LICENSE /app/LICENSE
COPY THIRD_PARTY_NOTICES.md /app/THIRD_PARTY_NOTICES.md
COPY migrations/ /app/migrations/
ENV DDNSUPD_WEB_DIR=/app/web
ENV DDNSUPD_MIGRATIONS_DIR=/app/migrations
USER 10001:10001
EXPOSE 8080
HEALTHCHECK --interval=15s --timeout=5s --start-period=20s --retries=3 CMD ["/app/ddnsupd", "healthcheck"]
ENTRYPOINT ["/app/ddnsupd"]
