FROM rustlang/rust:nightly AS builder

WORKDIR /app

RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk

COPY . .

RUN trunk build --release

# Serve static files
FROM nginx:alpine

COPY --from=builder /app/dist /usr/share/nginx/html