# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-02-07

### Added

#### Object Operations (14 APIs)
- `put_object` — Upload an object (with metadata, ACL, storage class)
- `get_object` — Download an object (with Range support)
- `head_object` — Get object metadata
- `delete_object` — Delete an object
- `delete_multiple_objects` — Batch delete (up to 1000 objects, quiet mode)
- `copy_object` — Copy an object (with metadata directive)
- `list_objects_v2` — List objects (prefix, delimiter, continuation token)
- `append_object` — Append data to an appendable object
- `restore_object` — Restore an archived object
- `get_object_acl` / `put_object_acl` — Get/Set object ACL
- `get_object_tagging` / `put_object_tagging` / `delete_object_tagging` — Object tagging

#### Bucket Operations (5 APIs)
- `create_bucket` — Create a bucket (with optional storage class)
- `delete_bucket` — Delete a bucket
- `list_buckets` — List all buckets
- `get_bucket_info` — Get bucket metadata and configuration
- `get_bucket_location` — Get bucket data center location

#### Multipart Upload (6 APIs)
- `initiate_multipart_upload` — Start a multipart upload
- `upload_part` — Upload a single part
- `complete_multipart_upload` — Finalize the upload
- `abort_multipart_upload` — Cancel and clean up
- `list_parts` — List uploaded parts
- `list_multipart_uploads` — List active multipart uploads

#### Presigned URLs
- `presign_get_object` — Generate download URL (V4 query-string signing)
- `presign_put_object` — Generate upload URL (V4 query-string signing)

#### High-Level APIs
- `TransferManager` — Automatic multipart upload with concurrent part uploads, CRC64 checksum verification, and progress tracking
- `ListObjectsV2Paginator` — Auto-paginated object listing
- `ListBucketsPaginator` — Auto-paginated bucket listing

#### Infrastructure
- V4 signature authentication (full implementation)
- Credential providers: `StaticProvider`, `EnvironmentProvider`, `ProviderChain`, `CachingProvider`
- STS temporary credential support
- Automatic retry with exponential backoff and jitter
- CRC64-ECMA checksum calculation and verification
- Request interceptor middleware chain
- Progress tracking via `ProgressListener` trait
- Strong types: `BucketName`, `ObjectKey`, `Region`, `StorageClass`, `ObjectAcl`
- Input validation for bucket names, object keys, metadata keys, part numbers, expiry durations
- Credential security: zeroize on drop, redacted `Debug` output, HTTPS enforced by default

[0.1.0]: https://github.com/infinitete/rs-ali-oss/releases/tag/v0.1.0
