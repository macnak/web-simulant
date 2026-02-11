# Multi-stage build for web-simulant
# Stage 1: Build
FROM rust:latest AS builder

WORKDIR /app

# Copy project files
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY static ./static
COPY examples ./examples

# Build release binary
RUN cargo build --release

# Stage 2: Runtime
FROM alpine:3.19

# Install runtime dependencies (minimal)
RUN apk add --no-cache \
    ca-certificates \
    curl

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/web-simulant /app/web-simulant

# Copy static files and example configs
COPY --from=builder /app/static /app/static
COPY --from=builder /app/examples /app/examples

# Create config directory for persistence
RUN mkdir -p /app/config

# Expose ports
# 8080: Engine (simulated API endpoints)
# 8081: Control Plane (admin UI and config management)
EXPOSE 8080 8081

# Health check for control plane
HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8081/api/health || exit 1

# Run the application
ENTRYPOINT ["/app/web-simulant"]
