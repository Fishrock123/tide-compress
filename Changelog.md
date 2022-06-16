# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- Feat: `Content-Type` header compressibility check via `Regex`.

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
