# metta-kg (Rust API + embedded UI)

Single-binary server that embeds the frontend build and serves API + UI via Rocket.

Build locally:

- Ensure Node.js 20+
- cd ../frontend && npm ci && npm run build
- cd ../api && cargo run -- --address 127.0.0.1 --port 3030

Release build:

- cd api && cargo build --release

Flags: see top-level README.
