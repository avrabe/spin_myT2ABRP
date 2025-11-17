# Multi-stage Dockerfile for MyT2ABRP
# Builds Rust components and packages everything for deployment

# Stage 1: Rust builder
FROM rust:1.75 as rust-builder

# Install wasm32-wasip2 target
RUN rustup target add wasm32-wasip2

# Set working directory
WORKDIR /app

# Copy web-ui component
COPY web-ui ./web-ui
COPY Cargo.toml .

# Build web-ui component
WORKDIR /app/web-ui
RUN cargo build --target wasm32-wasip2 --release

# Stage 2: Spin runtime
FROM ghcr.io/fermyon/spin:v2.7.0

# Copy built WASM components
COPY --from=rust-builder /app/web-ui/target/wasm32-wasip2/release/web_ui.wasm /app/web-ui/target/wasm32-wasip2/release/

# Copy Bazel-built main component (if it exists)
# Note: This should be built separately or in a different stage
COPY bazel-bin/myt2abrp_app.wasm /app/bazel-bin/ 2>/dev/null || true

# Copy Spin configuration
COPY spin.toml /app/

# Copy static files
COPY web-ui/static /app/web-ui/static

# Set working directory
WORKDIR /app

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run Spin
CMD ["spin", "up", "--listen", "0.0.0.0:3000"]
