FROM rust:1-trixie AS build

WORKDIR /app

COPY Cargo.* ./
COPY . ./

RUN cargo build --release

FROM debian:trixie-slim AS app

RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=build /app/target/release/cineco_cal /

EXPOSE 8000

ENTRYPOINT ["/cineco_cal"]