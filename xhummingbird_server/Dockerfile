FROM rust:1.49.0 as planner
WORKDIR app
RUN cargo install cargo-chef 
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM rust:1.49.0 as cacher
RUN apt-get update && apt-get install -y libzmq3-dev && apt-get clean
WORKDIR app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:1.49.0 as builder
RUN apt-get update && apt-get install -y libzmq3-dev && apt-get clean
WORKDIR app
COPY . .
COPY --from=cacher /app/target target
RUN cargo build --release --bin xhummingbird_server

FROM rust:1.49.0 as runtime
RUN apt-get update && apt-get install -y libzmq3-dev && apt-get clean
WORKDIR app
COPY . .
COPY --from=builder /app/target/release/xhummingbird_server /app
ENTRYPOINT ["./xhummingbird_server"]
