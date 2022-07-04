# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.10.4] - 2022-07-04

- Build: avoid including build deps if `"db-check"` feature is not enabled.
- Docs: minor formatting fixes.

## [0.10.3] - 2022-06-29

- Docs: minor updates, linking, readme badges, etc
- Meta: attempt to fix doc.rs build.

## [0.10.2] - 2022-06-20

- Deps: uses `h1-client-rustls` for Surf in build-deps
    - Avoids some compilation dependency issues on some platforms
- Deps: reduced dependance on http-types features

## [0.10.1] - 2022-06-17

- Docs: fix mention of outdated body size threshold configuration.

## [0.10.0] - 2022-06-17

- Feat: `Content-Type` header compressibility checking.
    - First via a check to [jshttp's comprehensive database](https://github.com/jshttp/mime-db/blob/master/db.json), which is compiled to a [perfect hash function](https://github.com/rust-phf/rust-phf) at build time.
    - Falls back to a `Regex` check, which is customizable.

## [0.9.0] - 2021-01-29

- Enabled `DEFLATE` support by default, since there is no real drawbacks to doing so.
    - Is the same algorithm and dependency as used for `GZIP`, just with no meta-info.
- Dependencies: updated to Tide 0.16

## [0.8.1] - 2020-11-19

- Dependencies: do not require any Tide features
    - This should prevent the default Tide logger from being included when using this crate.

## [0.8.0] - 2020-11-18

- Dependencies: updated to Tide 0.15

## [0.7.0] - 2020-10-17

- Added `Vary` header modification.
- Docs: various updates, more examples.
- Dependencies: updated to Tide 0.14
- Dependencies: now depends on `futures-lite` rather than `futures-util`.

## [0.6.0] - 2020-09-28

- Dependencies: updated for http-types 2.5
- Internal improvements: now uses http-types for Content-Encoding & Cache-Control.

## [0.5.0] - 2020-08-01

- Dependencies: updated for Tide 0.13
- Fixed/Improved `Content-Encoding` parsing.

## [0.4.0] - 2020-07-18

- Dependencies: updated for Tide 0.12

## [0.3.0] - 2020-06-05

- Dependencies: updated for tide 0.10

## [0.2.0] - 2020-05-26

- Added support for handling `Cache-Control: no-transform`.

## [0.1.1] - 2020-05-23

- Updated / fixed documentation.

## [0.1.0] - 2020-05-23

- Initial release.
- Support for Brotli, Gzip, and Deflate encodings, compile-time configurable through cargo feature flags.
  - Prioritizes Brotli if available.
  - Only pulls in the necessary dependencies for the desired configuration.
  - Defaults to Brotli + Gzip.
- `Accept-Encoding` checking.
- Minimum body size threshold.
  - Configurable when created by `CompressMiddleware::with_threshold(usize)`.
