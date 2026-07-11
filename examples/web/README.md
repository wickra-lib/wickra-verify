# Wickra Verify — browser demo

A tiny static page that runs the verifier in your browser via WebAssembly: paste
a claim, press **Verify**, and see it confirmed or refuted. The deterministic
Wickra engine recomputes the report from the strategy and data, so a doctored
`claimed_report` cannot pass.

**Static only — no server, no upload, no build backend.** Everything runs
locally in your browser; nothing you type leaves the page. There is no network
request to any API.

## Build and run

Build the WebAssembly package into this directory, then serve the folder over
HTTP (browsers refuse to load ES modules from `file://`):

```sh
# 1. Build the wasm package (once; requires wasm-pack)
( cd bindings/wasm && wasm-pack build --target web --out-dir ../../examples/web/pkg )

# 2. Serve this directory with any static file server, e.g.
python -m http.server -d examples/web
#    then open http://localhost:8000
```

The generated `pkg/` directory is a build artifact and is not committed.

## What it shows

The page loads with an example claim whose `fees_paid` has been inflated to
`99999.0`. Pressing **Verify** returns a `Verdict` with `matches: false` and the
`fees_paid` mismatch — the tamper is caught. Correct that number (or change any
field of the strategy, data, or report) and verify again to watch the verdict
move. A malformed claim surfaces an in-band `{ok:false,error:...}` message.

## Files

- `index.html` — the page and styling.
- `app.js` — loads the wasm verifier, prefills the example claim, and wires the
  **Verify** button to `Verifier.command({cmd:"verify", claim})`.
