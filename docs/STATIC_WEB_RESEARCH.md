# Spin Static File Serving Research

## Spin Static Fileserver Component

Spin provides a built-in `static-fileserver` component for serving static assets.

### Method 1: Using spin-fileserver Template

```bash
# Create static fileserver from template
spin new http-static my-static-site
```

### Method 2: Manual Configuration in spin.toml

```toml
[[trigger.http]]
route = "/static/..."
component = "fileserver"

[component.fileserver]
source = { url = "https://github.com/fermyon/spin-fileserver/releases/download/v0.3.0/spin_static_fs.wasm", digest = "sha256:..." }
files = [{ source = "assets", destination = "/" }]
```

### Method 3: Custom Rust Component

Build a custom component that serves files:

```rust
use spin_sdk::http::{IntoResponse, Request, ResponseBuilder};
use spin_sdk::http_component;

#[http_component]
fn handle_request(req: Request) -> anyhow::Result<impl IntoResponse> {
    let path = req.path();

    // Serve index.html for root
    if path == "/" || path == "/index.html" {
        return Ok(ResponseBuilder::new(200)
            .header("content-type", "text/html")
            .body(include_str!("../static/index.html"))
            .build());
    }

    // Serve other assets
    match path {
        "/styles.css" => Ok(ResponseBuilder::new(200)
            .header("content-type", "text/css")
            .body(include_str!("../static/styles.css"))
            .build()),
        "/app.js" => Ok(ResponseBuilder::new(200)
            .header("content-type", "application/javascript")
            .body(include_str!("../static/app.js"))
            .build()),
        _ => Ok(ResponseBuilder::new(404)
            .body("Not found")
            .build())
    }
}
```

## HTMX Integration

HTMX allows building dynamic interfaces with minimal JavaScript:

```html
<!DOCTYPE html>
<html>
<head>
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>
</head>
<body>
    <!-- Dynamic content loaded via HTMX -->
    <div hx-get="/api/status"
         hx-trigger="every 2s"
         hx-swap="innerHTML">
        Loading...
    </div>
</body>
</html>
```

## Architecture Decision

**Best Approach for MyT2ABRP:**
1. Use custom Rust component for flexibility
2. Embed static files at compile time (include_str!)
3. Serve API and static files from same component
4. Use HTMX for dynamic updates without SPA complexity

### Benefits:
- Single WASM component (simpler deployment)
- No external dependencies for static files
- Fast serving (files embedded in WASM)
- Full control over routing and caching

### Structure:
```
web-component/
├── src/
│   └── lib.rs              # HTTP handler
├── static/
│   ├── index.html          # Main dashboard
│   ├── styles.css          # Styling
│   ├── app.js              # Minimal JS (if needed)
│   └── htmx.min.js         # HTMX library (self-hosted)
└── Cargo.toml
```

## Implementation Plan

1. Create new Spin component `web-ui`
2. Embed static HTML/CSS/JS files
3. Serve index.html at `/`
4. API endpoints at `/api/*`
5. HTMX for dynamic updates
6. WebSocket support for real-time updates (optional)

## HTMX Features for Dashboard

- **hx-get**: Fetch vehicle status
- **hx-post**: Update charging settings
- **hx-trigger**: Polling for updates
- **hx-swap**: Update DOM sections
- **hx-indicator**: Loading states
- **Server-Sent Events**: Real-time push updates
