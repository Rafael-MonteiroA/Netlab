FROM rust:1.98-bookworm AS build
WORKDIR /src
COPY . .
RUN cargo build --locked --release -p netlab-server

FROM debian:bookworm-slim
RUN useradd --uid 10001 --create-home netlab
COPY --from=build /src/target/release/netlab-server /usr/local/bin/netlab-server
USER 10001
EXPOSE 8787
ENTRYPOINT ["netlab-server"]
