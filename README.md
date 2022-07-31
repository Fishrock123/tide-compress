[![tide-compress on crates.io](https://img.shields.io/crates/v/tide-compress)](https://crates.io/crates/tide-compress) [![Documentation (latest release)](https://docs.rs/tide-compress/badge.svg)](https://docs.rs/tide-compress/)

# tide-compress

Outgoing body compression middleware for the [Tide][] server framework.

```rust
#[async_std::main]
async fn main() {
    let mut app = tide::new();
    app.with(tide_compress::CompressMiddleware::new());
}
```

## Features

- Support for [Brotli][], [Gzip][], and [Deflate][] encodings, compile-time configurable through cargo feature flags.
  - Prioritizes Brotli if available.
  - Only pulls in the necessary dependencies for the desired configuration.
  - Defaults to Brotli & Gzip.
  - Also handles the `"identity"` encoding directive [as per RFC 9110][Identity].
- [`Accept-Encoding`][] header checking including priority.
- Minimum body size threshold (Default: 1024 bytes, configurable).
- Does not compress responses with a [`Cache-Control: no-transform`][] header.
- Sets the [`Vary`][] header.
- Checks the [`Content-Type`][] header (MIME).
  - Checks against [jshttp's comprehensive database][jshttp mime-db], which is compiled to a [perfect hash function][] at build time.
  - If not in the database, checks against a regular expression.
    - Default: `^text/|\+(?:json|text|xml)$` (case insensitive).
    - Fully override-able to any custom [`Regex`][], with `None` as an option.
  - Functionality can be excluded in crate features if the `regex` crate or codegen poses build issues.

## Note

This crate, in its current set up with the `db-check` feature enabled (which is by default),
pulls down [a json MIME database][jshttp mime-db] from the network at build time.

## License

Licensed under the [BlueOak Model License 1.0.0](LICENSE.md) — _[Contributions via DCO 1.1](contributing.md#developers-certificate-of-origin)_

[`Accept-Encoding`]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Encoding
[`Cache-Control: no-transform`]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control
[`Content-Type`]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Type
[`Regex`]: https://docs.rs/regex/1/regex/struct.Regex.html
[`Vary`]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Vary
[jshttp mime-db]: https://github.com/jshttp/mime-db/blob/master/db.json
[perfect hash function]: https://github.com/rust-phf/rust-phf
[Brotli]: https://en.wikipedia.org/wiki/Brotli
[Deflate]: https://en.wikipedia.org/wiki/Deflate
[Gzip]: https://en.wikipedia.org/wiki/Gzip
[Identity]: https://www.rfc-editor.org/rfc/rfc9110.html#name-accept-encoding
[Tide]: https://github.com/http-rs/tide
