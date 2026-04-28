# wasm-engine

Wasmtime-based runtime that executes WASM/WASI modules dispatched by [`wasm-operator`](https://github.com/pleme-io/wasm-operator).

## What it does

The engine is the entrypoint of every pod the operator spawns. It:

1. Reads `PROGRAM_MODULE_URL` from env and parses it with the URL grammar from [`theory/WASM-PACKAGING.md`](https://github.com/pleme-io/theory/blob/main/WASM-PACKAGING.md).
2. Fetches the module (skeleton stage — production fetch+cache pipeline lands in M1).
3. Initializes a wasmtime engine + WASI store with capability tokens parsed from `PROGRAM_CAPABILITIES`.
4. Dispatches according to `PROGRAM_TRIGGER`:

   | Trigger | Behavior |
   |---------|----------|
   | `service` | Runs an axum HTTP server on `PROGRAM_SERVICE_PORT` (default `8080`). |
   | `watch`/`event` | Long-running. Attaches an informer or event subscription. |
   | `cron`/`oneShot` | Executes the module once and exits. |

## Build

```bash
nix develop                  # devShell with rustc/cargo
cargo check                  # type-check
cargo build --release        # native binary
nix build .#default           # nix-built binary
nix build .#image             # Docker image (Linux target)
nix flake check --no-build    # evaluate flake
```

The Docker image is published to `ghcr.io/pleme-io/wasm-engine:<tag>` by the forge release pipeline.

## Run locally

```bash
PROGRAM_MODULE_URL='github:pleme-io/programs/hello-world/main.tlisp?ref=v0.1.0' \
PROGRAM_TRIGGER=service \
PROGRAM_SERVICE_PORT=8080 \
PROGRAM_CAPABILITIES='http-in:0.0.0.0:8080
kube-downward-api' \
cargo run --release
```

## License

MIT.
