FROM rustlang/rust:nightly

ENV DATA_PATH=./data
ENV HOST=0.0.0.0
ENV PORT=8080

WORKDIR /app
EXPOSE 8080
COPY . .

RUN cargo build --release
CMD ./target/release/uqoin-node
