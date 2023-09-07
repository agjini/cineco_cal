FROM rust:1.72 AS build

WORKDIR /app

COPY ./Cargo.* /app/
COPY ./src ./src

RUN cargo build --release

FROM debian:buster-slim AS app

COPY --from=build /app/target/release/cineco_cal /

CMD ["/cineco_cal"]