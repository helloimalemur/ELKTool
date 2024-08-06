FROM rust:1.79
COPY . .
RUN cargo install --path crates/core
CMD ["elktool-core"]
