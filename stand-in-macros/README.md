# stand-in-macros

[![Crates.io](https://img.shields.io/crates/v/stand-in-macros.svg)](https://crates.io/crates/stand-in-macros)
[![Docs.rs](https://docs.rs/stand-in-macros/badge.svg)](https://docs.rs/stand-in-macros)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Procedural macros for **[stand-in](https://crates.io/crates/stand-in)**.

This crate provides the attribute macros that power the `stand-in` MCP server framework:

- `#[mcp_tool]` — declare a tool; the JSON Schema is inferred from the function signature.
- `#[mcp_prompt]` — declare a reusable prompt template with typed arguments.
- `#[mcp_resource]` — expose data as a concrete or template (`{param}`) MCP resource.
- `#[mcp_server]` — generate initialization, capability negotiation, and JSON-RPC dispatch.

They generate the JSON-RPC dispatch, capability advertisement, and handler wiring at
compile time, and register each item via the [`inventory`](https://crates.io/crates/inventory)
pattern.

You normally **don't depend on this crate directly** — add
[`stand-in`](https://crates.io/crates/stand-in), which re-exports every macro through its
prelude:

```rust
use stand_in::prelude::*;
```

See the [stand-in](https://crates.io/crates/stand-in) crate for usage and the complete
framework.

## License

MIT
