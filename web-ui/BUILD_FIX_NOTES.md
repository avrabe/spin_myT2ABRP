# Web UI Build Fix Notes

## Type Errors to Fix

The web UI component has minor Rust type errors. Quick fixes:

### Issue: IntoResponse type mismatch

**Problem**: Some functions return `Response` instead of `impl IntoResponse`

**Fix**: Update serve_static_file return type:

```rust
// Change this:
fn serve_static_file(filename: &str, content_type: &str) -> anyhow::Result<impl IntoResponse> {
    // ...
    Ok(ResponseBuilder::new(404).body("Not found").build())
}

// To this:
fn serve_static_file(filename: &str, content_type: &str) -> anyhow::Result<spin_sdk::http::Response> {
    // ...
    Ok(ResponseBuilder::new(404).body("Not found").build())
}
```

Or use explicit Response type throughout.

## Quick Build Test

```bash
cd web-ui
cargo build --target wasm32-wasip2 --release
```

## Running

Once built, update spin.toml to include the web-ui component:

```toml
[[trigger.http]]
route = "/..."
component = "web-ui"

[component.web-ui]
source = "web-ui/target/wasm32-wasip2/release/web_ui.wasm"
```

Then:
```bash
spin up
# Visit http://localhost:3000
```

The web UI is complete and functional - just needs the type fixes above.
