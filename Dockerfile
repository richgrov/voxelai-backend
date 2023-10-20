FROM rust:1.70
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path . && cargo clean
ENTRYPOINT ["constructor"]

