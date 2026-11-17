FROM rust:trixie
RUN cargo install cargo-audit --version 0.22.2 --locked
