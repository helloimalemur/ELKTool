FROM rust:1.79
COPY . .
RUN cargo install --path crates/sanitize
CMD ["elktool-sanitize"]
