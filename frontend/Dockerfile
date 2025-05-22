FROM rustlang/rust:nightly AS builder

WORKDIR /app

RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk wasm-bindgen-cli

COPY frontend ./frontend
WORKDIR /app/frontend

RUN trunk build --release

FROM nginx:alpine
COPY --from=builder /app/frontend/dist /usr/share/nginx/html