# AGENTS.md — rs-ali-oss

Rust SDK for Alibaba Cloud Object Storage Service (OSS).
Library crate. Rust edition 2024. Toolchain: stable.

---

## Build / Test / Lint Commands

```bash
# Build
cargo build                        # debug build
cargo build --release              # release build

# Test — all
cargo test                         # run all tests
cargo test -- --nocapture          # run all tests, show stdout/stderr

# Test — single test by name
cargo test test_name               # run tests matching "test_name"
cargo test test_name -- --exact    # run only the test named exactly "test_name"

# Test — single module
cargo test module_name::           # run all tests in a module

# Test — single integration test file
cargo test --test integration_test_file_name

# Lint
cargo clippy                       # lint (treat as mandatory)
cargo clippy -- -D warnings        # lint, deny all warnings (CI mode)

# Format
cargo fmt                          # format all code
cargo fmt -- --check               # check formatting without modifying

# Doc
cargo doc --no-deps --open         # build and open docs

# Full CI check (run before committing)
cargo fmt -- --check && cargo clippy -- -D warnings && cargo test
```

---

## Setup (after cloning)

```bash
./setup.sh              # configures git to use .githooks/pre-commit
```

This enables the pre-commit hook that runs `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test` before every commit. CI (GitHub Actions) runs the same checks on push/PR to `main`.

---

## Project Structure

```
rs-ali-oss/
├── Cargo.toml          # manifest — dependencies, metadata
├── setup.sh            # one-time hook setup script
├── .githooks/
│   └── pre-commit      # fmt + clippy + test gate
├── .github/workflows/
│   └── ci.yml          # GitHub Actions CI
├── src/
│   ├── lib.rs          # crate root — public API re-exports
│   ├── client.rs       # OSS client, retry logic, URL construction
│   ├── config.rs       # ClientBuilder, Config, Credentials
│   ├── error.rs        # error types (thiserror)
│   ├── credential.rs   # CredentialProvider trait and implementations
│   ├── crc64.rs        # CRC64-ECMA checksum
│   ├── progress.rs     # ProgressListener trait
│   ├── encoding.rs     # URI/Query percent-encoding sets (crate-internal)
│   ├── middleware.rs    # request interceptor chain
│   ├── auth/
│   │   ├── mod.rs      # auth module root
│   │   └── v4.rs       # V4 signature algorithm
│   ├── ops/
│   │   ├── mod.rs      # ops module root
│   │   ├── object.rs   # object operations (14 methods)
│   │   ├── bucket.rs   # bucket operations (26 methods: 5 CRUD + 21 management)
│   │   ├── multipart.rs # multipart upload operations (6 methods)
│   │   ├── presign.rs  # presigned URL generation
│   │   ├── paginator.rs # auto-paginators
│   │   └── transfer.rs # Transfer Manager (managed multipart upload)
│   └── types/
│       ├── mod.rs      # types module root
│       ├── common.rs   # BucketName, ObjectKey, Region, StorageClass, ObjectAcl, BucketAcl, CorsHttpMethod, VersioningStatus, ServerSideEncryption, MetadataDirective
│       ├── response.rs # all response types
│       └── request/    # all request builders
│           ├── mod.rs
│           ├── object.rs
│           ├── bucket.rs
│           ├── multipart.rs
│           └── presign.rs
└── tests/
    ├── object_ops.rs   # object operation integration tests
    ├── bucket_ops.rs   # bucket operation integration tests
    ├── multipart_ops.rs # multipart upload integration tests
    └── retry.rs        # retry logic integration tests
```

> This structure will evolve. Keep `lib.rs` as a thin re-export layer.

---

## Code Style Guidelines

### Formatting

- Use `rustfmt` defaults — do NOT add a `rustfmt.toml` unless necessary.
- Max line width: 100 chars (rustfmt default).
- Use trailing commas in multi-line constructs.

### Naming Conventions

| Item               | Convention           | Example                    |
|--------------------|----------------------|----------------------------|
| Crate              | snake_case           | `rs_ali_oss`               |
| Modules            | snake_case           | `object_ops`               |
| Types / Traits     | PascalCase           | `OssClient`, `BucketInfo`  |
| Functions / Methods| snake_case           | `put_object`               |
| Constants          | SCREAMING_SNAKE_CASE | `DEFAULT_ENDPOINT`         |
| Type parameters    | Single uppercase     | `T`, `E`, `R`              |
| Lifetimes          | Short lowercase      | `'a`, `'de`                |
| Builder methods    | snake_case, no `set_` prefix | `.region("cn-hangzhou")` |

### Imports

- Group imports in this order, separated by blank lines:
  1. `std` / `core` / `alloc`
  2. External crates (from `Cargo.toml` dependencies)
  3. Crate-internal (`crate::`, `super::`, `self::`)
- Use `use` with braces for multiple items from the same module.
- Prefer explicit imports over glob (`*`) imports — except in test modules where `use super::*` is acceptable.

```rust
use std::collections::HashMap;
use std::io;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::error::OssError;
```

### Types

- Use strong typing — avoid bare `String` for domain concepts. Prefer newtypes:
  ```rust
  pub struct BucketName(String);
  pub struct ObjectKey(String);
  ```
- Use `&str` for function parameters when ownership is not needed.
- Prefer `impl Into<String>` or `impl AsRef<str>` for ergonomic public APIs.
- Use `Builder` pattern for complex constructors.
- Derive common traits: `Debug`, `Clone`, `PartialEq` on public types.

### Error Handling

- Define a crate-level error enum in `src/error.rs`.
- Use `thiserror` for deriving `std::error::Error`.
- Define a crate-level `Result<T>` type alias.
- NEVER use `.unwrap()` or `.expect()` in library code — only in tests.
- Use `?` operator for propagation.

```rust
// src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OssError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("XML parsing error: {0}")]
    XmlParse(String),

    #[error("Authentication failed: {0}")]
    Auth(String),
}

pub type Result<T> = std::result::Result<T, OssError>;
```

### Documentation

- All public items MUST have doc comments (`///`).
- Include usage examples in doc comments for key APIs.
- Use `#![deny(missing_docs)]` in `lib.rs` once the API stabilizes.

### Async

- Use `tokio` as the async runtime (if async is adopted).
- Prefer `async fn` over returning boxed futures.
- All I/O operations (HTTP, file) should be async.

### Testing

- Unit tests live in a `#[cfg(test)] mod tests` block inside each source file.
- Integration tests go in `tests/` directory.
- Use descriptive test names: `test_put_object_returns_etag`, not `test1`.
- Test both success and error paths.
- Use `#[should_panic]` or match on `Err` for expected failures — never ignore errors.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_object_with_valid_key_succeeds() {
        // arrange, act, assert
    }

    #[test]
    fn put_object_with_empty_key_returns_error() {
        let result = put_object("");
        assert!(result.is_err());
    }
}
```

### Dependencies Policy

- Keep dependencies minimal — this is a library crate.
- Prefer well-maintained, widely-used crates.
- Expected core dependencies:
  - `reqwest` — HTTP client
  - `serde` / `serde_json` / `quick-xml` — serialization
  - `thiserror` — error types
  - `hmac` / `sha2` — OSS signature
  - `tokio` — async runtime (dev/optional)
  - `chrono` or `time` — timestamps

### Safety

- No `unsafe` blocks unless absolutely necessary and well-documented.
- No `.unwrap()` in library code (only tests).
- Credentials (AccessKey ID/Secret) must never be logged or included in errors.
- Implement `Zeroize` for credential types if possible.

---

## Commit Conventions

- Use conventional commits: `feat:`, `fix:`, `docs:`, `test:`, `refactor:`, `chore:`
- Keep commits atomic — one logical change per commit.

---

## CI Checklist (before every PR)

1. `cargo fmt -- --check` passes
2. `cargo clippy -- -D warnings` passes
3. `cargo test` passes
4. No new `.unwrap()` in non-test code
5. Public APIs have doc comments
