FROM rustlang/rust:nightly

ENV DATA_PATH=./data
ENV HOST=0.0.0.0
ENV PORT=5772

WORKDIR /app
EXPOSE 5772
COPY . .

RUN cargo build --release
CMD ./target/release/uqoin-node
