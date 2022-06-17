# tide-compress

Outgoing compression middleware for the [Tide][] server framework.

```rust
#[async_std::main]
async fn main() -> tide::Result {
    let mut app = tide::new();
    app.with(tide_compress::CompressMiddleware::new());
}
```

## Features

- Support for Brotli, Gzip, and Deflate encodings, compile-time configurable through cargo feature flags.
  - Prioritizes Brotli if available.
  - Only pulls in the necessary dependencies for the desired configuration.
  - Defaults to Brotli & Gzip.
- `Accept-Encoding` checking including priority.
- Minimum body size threshold (Default: 1024 bytes, configurable).
- Does not compress responses with a [`Cache-Control: no-transform`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control) header.
- Sets the [`Vary`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Vary) header.
- Checks the [`Content-Type`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Type) header (MIME).
  - Checks against [jshttp's comprehensive database](https://github.com/jshttp/mime-db/blob/master/db.json), which is compiled to a [perfect hash function](https://github.com/rust-phf/rust-phf) at build time.
  - If not in the database, checks against a regular expression.
    - Default: `^text/|\+(?:json|text|xml)$` (case insensitive).
    - Fully override-able to any custom [`Regex`](https://docs.rs/regex/1/regex/struct.Regex.html), with `None` as an option.
  - Functionality can be excluded in crate features if the `regex` crate or codegen poses build issues.

## License

Licensed under the [BlueOak Model License 1.0.0](LICENSE.md) — _[Contributions via DCO 1.1](contributing.md#developers-certificate-of-origin)_

[Tide]: https://github.com/http-rs/tide

