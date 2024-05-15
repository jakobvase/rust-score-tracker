FROM debian:bullseye-slim
COPY target/x86_64-unknown-linux-musl/release/rust-score-tracker /usr/local/bin/rust-score-tracker
COPY pages /app/pages
ENTRYPOINT ["rust-score-tracker"]