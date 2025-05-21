# ---- Build stage ----
FROM rustlang/rust:nightly AS builder

# Create app directory
WORKDIR /app

# Copy the full workspace into the container
COPY . .

# Ensure dependencies are cached first (if unchanged)
RUN cargo fetch

# Build only the backend binary, but include all workspace crates
RUN cargo build --release --package backend

# ---- Runtime stage ----
FROM debian:bookworm-slim AS runtime

# Create non-root user
RUN useradd -m appuser

# Create directory for app
WORKDIR /app

# Copy only the compiled backend binary from the builder stage
COPY --from=builder /app/target/release/backend /app/backend

# Copy the ml-cmd binary if needed at runtime
#COPY --from=builder /app/target/release/ml-cmd /usr/local/bin/ml-cmd

# Copy any static assets if needed (optional)
COPY ./static /app/static

# Set ownership and permissions
RUN chown -R appuser:appuser /app && chmod +x /app/backend

USER appuser

# Set the entrypoint to the backend binary
CMD ["./backend"]