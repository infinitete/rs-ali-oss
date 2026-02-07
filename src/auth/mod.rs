//! Authentication and request signing for Alibaba Cloud OSS.

pub mod v4;

pub use v4::sign_request;
