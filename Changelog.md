# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
