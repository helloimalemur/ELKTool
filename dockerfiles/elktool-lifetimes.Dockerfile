FROM rust:1.79
COPY . .
RUN cargo install --path crates/lifetimes
CMD ["elktool-lifetimes"]
