# Repository Guidelines

## Project Structure & Module Organization

- `src/` contains the Rust crate source.
  - `src/main.rs` wires env/config, logging/metrics, and starts the JSON-RPC server.
  - `src/rpc_server.rs` implements the `getPriorityFeeEstimate` method (+ `/health`).
  - `src/priority_fee*.rs` contains fee tracking and estimation logic.
  - `src/grpc_*.rs` integrates Yellowstone gRPC / Geyser consumption.
  - `src/solana/` wraps Solana RPC helpers.
- `.github/workflows/ci.yaml` runs `cargo test -- --nocapture` in CI.

## Build, Test, and Development Commands

- `cargo build` — compile the service.
- `RPC_URL=... GRPC_URL=... cargo run` — run locally (see env vars below).
- `cargo test -- --nocapture` — run unit tests with logs (matches CI).
- `cargo fmt --all` — format with rustfmt.
- `cargo clippy --all-targets -- -D warnings` — lint (recommended before PRs).

Toolchain is pinned in `rust-toolchain.toml` (Rust `1.92.0`); CI currently uses stable.

## Coding Style & Naming Conventions

- Prefer rustfmt defaults; keep diffs small and focused.
- Use `snake_case` for modules/functions, `CamelCase` for types, and `SCREAMING_SNAKE_CASE` for constants.
- Keep error handling explicit (`thiserror` for typed errors, `anyhow` at boundaries).

## Testing Guidelines

- Tests use Rust’s built-in test harness and are mostly colocated (`mod tests` in `src/*.rs`).
- Keep tests deterministic: avoid network/time dependencies.
- When changing estimation logic, add coverage for edge cases (empty slots, vote inclusion, lookback windows).

## Commit & Pull Request Guidelines

- Commit messages commonly use a ticket prefix (e.g. `RPC-470 ...`); docs/chores may use Conventional Commits (e.g. `chore(docs): ...`). Keep summaries imperative and under ~72 chars.
- PRs should follow `.github/pull_request_template.md` and include:
  - what changed and why
  - how it was tested (paste command(s) run)
  - `README.md` updates when API behavior or options change

## Configuration & Security Notes

- Required runtime env (see `README.md`): `RPC_URL`, `GRPC_URL`, and optional `GRPC_X_TOKEN`.
- Optional: `PORT` (default `4141`), `MAX_LOOKBACK_SLOTS` (default `150`), `RUST_LOG`, `METRICS_URI`, `METRICS_PORT`.
- Never commit real RPC tokens/credentials; keep secrets out of logs.
