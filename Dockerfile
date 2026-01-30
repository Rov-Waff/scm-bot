FROM library/rust:trixie AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM library/debian:trixie-slim AS final
WORKDIR /app
COPY --from=builder /app/target/release/scm-bot .
CMD [ "scm-bot" ]