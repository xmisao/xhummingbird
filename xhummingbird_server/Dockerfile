FROM rust:1.49.0
ADD . /app
WORKDIR /app
RUN apt-get update && apt-get install -y libzmq3-dev && apt-get clean
RUN cargo build --release
CMD cargo run --release
