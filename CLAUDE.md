# wasm-engine

> **★★★ CSE / Knowable Construction.** This repo operates under **Constructive Substrate Engineering** — canonical specification at [`pleme-io/theory/CONSTRUCTIVE-SUBSTRATE-ENGINEERING.md`](https://github.com/pleme-io/theory/blob/main/CONSTRUCTIVE-SUBSTRATE-ENGINEERING.md). The Compounding Directive (operational rules: solve once, load-bearing fixes only, idiom-first, models stay current, direction beats velocity) is in the org-level pleme-io/CLAUDE.md ★★★ section. Read both before non-trivial changes.

Wasmtime-based runtime image used by [`wasm-operator`](https://github.com/pleme-io/wasm-operator)-spawned pods as the entrypoint. Pulls a `.wasm` / `.tlisp` module via the URL grammar from [`theory/WASM-PACKAGING.md`](https://github.com/pleme-io/theory/blob/main/WASM-PACKAGING.md) and dispatches it according to `PROGRAM_TRIGGER`.

## Repo orientation

| Path | Purpose |
|------|---------|
| `src/main.rs` | tokio runtime; dispatches per `PROGRAM_TRIGGER` (`service`/`watch`/`event`/`cron`/`oneShot`). |
| `src/url.rs` | URL grammar parser (`github:owner/repo/path?ref=tag`, `oci://…`, `https://…#blake3=…`, …). |
| `src/wasi.rs` | WASI HTTP server boilerplate. Skeleton axum router on port 8080. |
| `src/capabilities.rs` | Capability dispatch table — token parser + future host-import gating. |
| `flake.nix` | Consumes `substrate/lib/rust-service-flake.nix` with `moduleDir = null`, `nixosModuleFile = null`, and the `module = { description = …; }` trio. Builds the Docker image. |

## Environment contract

The operator populates these env vars on the spawned pod:

| Env var                | From `Program.spec.…`     |
|------------------------|--------------------------|
| `PROGRAM_MODULE_URL`   | `module.source`          |
| `PROGRAM_MODULE_BLAKE3`| `module.blake3` (opt.)   |
| `PROGRAM_TRIGGER`      | `trigger.kind`           |
| `PROGRAM_CONFIG`       | JSON-encoded `config`    |
| `PROGRAM_CAPABILITIES` | newline-joined `capabilities` |
| `PROGRAM_SERVICE_PORT` | `trigger.service.port` (service shape only) |

## Build

```bash
nix flake check --no-build       # evaluate without building
nix build .#default               # native binary
nix build .#image                 # Docker image (Linux target; needs remote builder on Darwin)
```

## Theory references

- [`WASM-STACK.md`](https://github.com/pleme-io/theory/blob/main/WASM-STACK.md) — runtime design, capability-based security
- [`WASM-PACKAGING.md`](https://github.com/pleme-io/theory/blob/main/WASM-PACKAGING.md) — module URL grammar
- [`WASM-RUNTIME-COMPLETE.md`](https://github.com/pleme-io/theory/blob/main/WASM-RUNTIME-COMPLETE.md) — runtime closure (elasticity, container images, recursive bootstrap)
- [`CONSTRUCTIVE-SUBSTRATE-ENGINEERING.md`](https://github.com/pleme-io/theory/blob/main/CONSTRUCTIVE-SUBSTRATE-ENGINEERING.md) — methodology

## Status

Phase A — skeleton scaffolded. `cargo check` passes; `nix flake check` evaluates clean. Module URL parser + capability tokenizer have unit tests; trigger dispatcher logs and either serves an axum stub (`service` shape) or sleeps/exits. M1 deliverable: wasmtime engine wiring, module fetch+blake3-cache pipeline, WASI host-import gates against the capability set.
