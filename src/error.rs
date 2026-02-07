//! Error types for the Alibaba Cloud OSS SDK.

use std::time::Duration;

use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;

/// Errors that can occur when interacting with Alibaba Cloud OSS.
#[derive(Debug, Error)]
pub enum OssError {
    /// HTTP transport error.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// XML parsing error.
    #[error("XML parsing error: {0}")]
    XmlParse(String),

    /// OSS service returned an error response.
    #[error("OSS service error (HTTP {status}): {code} - {message}")]
    ServerError {
        /// HTTP status code.
        status: u16,
        /// OSS error code.
        code: String,
        /// Human-readable error message.
        message: String,
        /// Request ID for troubleshooting.
        request_id: String,
        /// Host that generated the error.
        host_id: String,
    },

    /// Invalid bucket name.
    #[error("invalid bucket name: {0}")]
    InvalidBucketName(String),

    /// Invalid object key.
    #[error("invalid object key: {0}")]
    InvalidObjectKey(String),

    /// Invalid region.
    #[error("invalid region: {0}")]
    InvalidRegion(String),

    /// Authentication or signing error.
    #[error("authentication error: {0}")]
    Auth(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Missing required field in builder.
    #[error("missing required field: {0}")]
    MissingField(String),

    /// Operation timed out.
    #[error("operation timed out after {0:?}")]
    Timeout(Duration),

    /// Invalid parameter value.
    #[error("invalid parameter `{field}`: {reason}")]
    InvalidParameter {
        /// The parameter name.
        field: String,
        /// Why the value is invalid.
        reason: String,
    },

    /// All retry attempts exhausted.
    #[error("retry exhausted after {attempts} attempt(s)")]
    RetryExhausted {
        /// Number of attempts made.
        attempts: u32,
        /// The last error encountered.
        last_error: Box<OssError>,
    },

    /// Invalid URL construction.
    #[error("invalid URL: {0}")]
    InvalidUrl(String),
}

/// A specialized `Result` type for OSS operations.
pub type Result<T> = std::result::Result<T, OssError>;

/// Raw OSS error response XML structure.
#[derive(Debug, Deserialize)]
#[serde(rename = "Error")]
struct OssErrorResponse {
    #[serde(rename = "Code")]
    code: String,
    #[serde(rename = "Message")]
    message: String,
    #[serde(rename = "RequestId")]
    request_id: String,
    #[serde(rename = "HostId", default)]
    host_id: String,
}

impl OssError {
    /// Parse an OSS error response from HTTP status and body.
    ///
    /// Attempts to parse the body as OSS XML error format. Falls back to
    /// a raw message if XML parsing fails.
    pub fn from_response_body(status: StatusCode, body: &str) -> Self {
        match quick_xml::de::from_str::<OssErrorResponse>(body) {
            Ok(err_resp) => OssError::ServerError {
                status: status.as_u16(),
                code: err_resp.code,
                message: err_resp.message,
                request_id: err_resp.request_id,
                host_id: err_resp.host_id,
            },
            Err(_) => OssError::ServerError {
                status: status.as_u16(),
                code: String::new(),
                message: body.to_string(),
                request_id: String::new(),
                host_id: String::new(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_xml_error() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Error>
    <Code>NoSuchKey</Code>
    <Message>The specified key does not exist.</Message>
    <RequestId>534B371674E88A4D8906XXXX</RequestId>
    <HostId>my-bucket.oss-cn-hangzhou.aliyuncs.com</HostId>
</Error>"#;
        let err = OssError::from_response_body(StatusCode::NOT_FOUND, xml);
        match err {
            OssError::ServerError {
                status,
                code,
                message,
                request_id,
                ..
            } => {
                assert_eq!(status, 404);
                assert_eq!(code, "NoSuchKey");
                assert_eq!(message, "The specified key does not exist.");
                assert_eq!(request_id, "534B371674E88A4D8906XXXX");
            }
            other => panic!("expected ServerError, got: {other:?}"),
        }
    }

    #[test]
    fn parse_malformed_xml_falls_back() {
        let body = "not xml at all";
        let err = OssError::from_response_body(StatusCode::INTERNAL_SERVER_ERROR, body);
        match err {
            OssError::ServerError {
                status,
                message,
                code,
                ..
            } => {
                assert_eq!(status, 500);
                assert_eq!(message, "not xml at all");
                assert!(code.is_empty());
            }
            other => panic!("expected ServerError fallback, got: {other:?}"),
        }
    }

    #[test]
    fn display_formats_correctly() {
        let err = OssError::InvalidBucketName("AB".to_string());
        assert_eq!(err.to_string(), "invalid bucket name: AB");

        let err = OssError::Auth("signature mismatch".to_string());
        assert_eq!(err.to_string(), "authentication error: signature mismatch");
    }

    #[test]
    fn io_error_converts() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let oss_err: OssError = io_err.into();
        assert!(matches!(oss_err, OssError::Io(_)));
    }

    #[test]
    fn display_invalid_parameter() {
        let err = OssError::InvalidParameter {
            field: "expires".to_string(),
            reason: "must be at least 1 second".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "invalid parameter `expires`: must be at least 1 second"
        );
    }

    #[test]
    fn display_retry_exhausted() {
        let inner = OssError::Auth("signature mismatch".to_string());
        let err = OssError::RetryExhausted {
            attempts: 4,
            last_error: Box::new(inner),
        };
        assert_eq!(err.to_string(), "retry exhausted after 4 attempt(s)");
    }

    #[test]
    fn display_invalid_url() {
        let err = OssError::InvalidUrl("missing scheme".to_string());
        assert_eq!(err.to_string(), "invalid URL: missing scheme");
    }
}
