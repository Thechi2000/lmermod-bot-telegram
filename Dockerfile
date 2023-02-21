FROM debian:bullseye AS runtime
RUN apt-get update
RUN apt-get upgrade -y
RUN apt-get install -y ca-certificates

FROM rust:latest AS build

RUN cargo install cargo-build-deps

RUN USER=root cargo new --bin lmermod-bot-telegram
WORKDIR /lmermod-bot-telegram

COPY Cargo.toml Cargo.lock ./
RUN cargo build-deps --release

COPY ./src ./src
RUN cargo build  --release

FROM runtime
WORKDIR /var/bot
COPY --from=build /lmermod-bot-telegram/target/release/lmermod-bot-telegram .
CMD ["./lmermod-bot-telegram"]
