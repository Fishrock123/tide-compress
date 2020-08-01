# tide-compress

Outgoing compression middleware for the [Tide][] web framework.

```rust
#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    let mut app = tide::new();
    app.with(tide_compress::CompressMiddleware::new());
}
```

## Features

- Support for Brotli, Gzip, and Deflate encodings, compile-time configurable through cargo feature flags.
  - Prioritizes Brotli if available.
  - Only pulls in the necessary dependencies for the desired configuration.
  - Defaults to Brotli + Gzip.
- `Accept-Encoding` checking.
- Minimum body size threshold.
  - Configurable when created by `CompressMiddleware::with_threshold(usize)`.
- Does not compress responses with a [`Cache-Control: no-transform`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control) header.

## Caveats

- Does not do `Accept-Encoding` priority handling.
- Does not do any `Content-Type` / MIME checking.
- Does not adjust the `Vary` header.

## License

Licensed under the [BlueOak Model License 1.0.0](LICENSE) — _[Contributions via DCO 1.1](contributing.md#developers-certificate-of-origin)_

[Tide]: https://github.com/http-rs/tide

