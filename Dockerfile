FROM rust:1.72 AS build

WORKDIR /app

COPY ./Cargo.* /app/
COPY ./src ./src

RUN cargo build --release

FROM debian:12-slim AS app

COPY --from=build /app/target/release/cineco_cal /

ENV ROCKET_ADDRESS=0.0.0.0

EXPOSE 8000

CMD ["/cineco_cal"]