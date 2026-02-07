//! Object operations: PutObject, GetObject, DeleteObject, HeadObject, ListObjectsV2, CopyObject.

use std::collections::HashMap;

use base64::Engine;
use md5::{Digest, Md5};
use percent_encoding::utf8_percent_encode;
use reqwest::Method;

use crate::client::{
    OssClient, header_etag, header_etag_opt, header_opt, parse_xml, serialize_xml,
};
use crate::encoding::URI_ENCODE_SET;
use crate::error::Result;
use crate::types::request::{
    AppendObjectRequest, CopyObjectRequest, DeleteMultipleObjectsRequest, DeleteMultipleObjectsXml,
    DeleteObjectRequest, DeleteObjectTaggingRequest, DeleteObjectXmlEntry, GetObjectAclRequest,
    GetObjectRequest, GetObjectTaggingRequest, HeadObjectRequest, ListObjectsV2Request,
    PutObjectAclRequest, PutObjectRequest, PutObjectTaggingRequest, RestoreObjectRequest,
};
use crate::types::response::{
    AppendObjectResponse, CopyObjectResponse, DeleteMultipleObjectsResponse, DeleteObjectResponse,
    DeleteObjectTaggingResponse, GetObjectAclResponse, GetObjectResponse, GetObjectTaggingResponse,
    HeadObjectResponse, ListObjectsV2Response, ObjectBody, PutObjectAclResponse, PutObjectResponse,
    PutObjectTaggingResponse, RestoreObjectResponse, Tag, TagSet, TaggingXml,
};

impl OssClient {
    /// Upload an object to OSS.
    ///
    /// # Payload Signing
    ///
    /// When the body is backed by in-memory bytes (e.g., `Vec<u8>`, `Bytes`),
    /// the SDK computes a SHA-256 hash of the payload and includes it in the
    /// V4 signature. When the body is a non-buffered stream, the SDK uses
    /// `UNSIGNED-PAYLOAD` — the request is still authenticated via the
    /// Authorization header, but the payload itself is not integrity-checked
    /// by the signature. OSS may still validate Content-MD5 or CRC64 if
    /// those headers are present.
    ///
    /// # Examples
    /// ```no_run
    /// # use rs_ali_oss::*;
    /// # use rs_ali_oss::types::request::PutObjectRequestBuilder;
    /// # async fn example(client: OssClient) -> Result<()> {
    /// let request = PutObjectRequestBuilder::new()
    ///     .bucket(BucketName::new("my-bucket")?)
    ///     .key(ObjectKey::new("hello.txt")?)
    ///     .body(b"Hello, OSS!".to_vec())
    ///     .content_type("text/plain")
    ///     .build()?;
    /// let response = client.put_object(request).await?;
    /// println!("ETag: {}", response.etag);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn put_object(&self, request: PutObjectRequest) -> Result<PutObjectResponse> {
        let url = self.build_url(Some(&request.bucket), Some(&request.key), &[])?;
        let mut http_req = self.http_client().request(Method::PUT, url);

        if let Some(ref ct) = request.content_type {
            http_req = http_req.header("content-type", ct.as_str());
        }
        if let Some(sc) = request.storage_class {
            http_req = http_req.header("x-oss-storage-class", sc.to_string());
        }
        if let Some(acl) = request.acl {
            http_req = http_req.header("x-oss-object-acl", acl.to_string());
        }
        for (k, v) in &request.metadata {
            http_req = http_req.header(format!("x-oss-meta-{k}"), v.as_str());
        }

        let http_req = http_req.body(request.body).build()?;
        let response = self.execute(http_req).await?;

        let etag = header_etag(&response);
        let request_id = header_opt(&response, "x-oss-request-id");

        Ok(PutObjectResponse { etag, request_id })
    }

    /// Download an object from OSS.
    ///
    /// Returns a streaming response — the body is NOT buffered in memory.
    pub async fn get_object(&self, request: GetObjectRequest) -> Result<GetObjectResponse> {
        let url = self.build_url(Some(&request.bucket), Some(&request.key), &[])?;
        let mut http_req = self.http_client().request(Method::GET, url);

        if let Some(ref range) = request.range {
            http_req = http_req.header("range", range.as_str());
        }

        let http_req = http_req.build()?;
        let response = self.execute(http_req).await?;

        let content_type = header_opt(&response, "content-type");
        let content_length = response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());
        let etag = header_etag_opt(&response);
        let request_id = header_opt(&response, "x-oss-request-id");

        Ok(GetObjectResponse {
            body: ObjectBody::new(response),
            content_type,
            content_length,
            etag,
            request_id,
        })
    }

    /// Delete an object from OSS.
    pub async fn delete_object(
        &self,
        request: DeleteObjectRequest,
    ) -> Result<DeleteObjectResponse> {
        let url = self.build_url(Some(&request.bucket), Some(&request.key), &[])?;
        let http_req = self.http_client().request(Method::DELETE, url).build()?;
        let response = self.execute(http_req).await?;

        let request_id = header_opt(&response, "x-oss-request-id");

        Ok(DeleteObjectResponse { request_id })
    }

    /// Retrieve object metadata without downloading the body.
    pub async fn head_object(&self, request: HeadObjectRequest) -> Result<HeadObjectResponse> {
        let url = self.build_url(Some(&request.bucket), Some(&request.key), &[])?;
        let http_req = self.http_client().request(Method::HEAD, url).build()?;
        let response = self.execute(http_req).await?;

        let content_type = header_opt(&response, "content-type");
        let content_length = response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());
        let etag = header_etag_opt(&response);
        let last_modified = header_opt(&response, "last-modified").and_then(|s| {
            chrono::DateTime::parse_from_rfc2822(&s)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .ok()
                .or_else(|| {
                    s.find(", ").and_then(|pos| {
                        chrono::NaiveDateTime::parse_from_str(
                            &s[pos + 2..],
                            "%d %b %Y %H:%M:%S GMT",
                        )
                        .ok()
                        .map(|dt| dt.and_utc())
                    })
                })
        });
        let request_id = header_opt(&response, "x-oss-request-id");

        let mut metadata = HashMap::new();
        for (name, value) in response.headers() {
            if let Some(meta_key) = name.as_str().strip_prefix("x-oss-meta-")
                && let Ok(v) = value.to_str()
            {
                metadata.insert(meta_key.to_string(), v.to_string());
            }
        }

        Ok(HeadObjectResponse {
            content_type,
            content_length,
            etag,
            last_modified,
            metadata,
            request_id,
        })
    }

    /// List objects in a bucket using the V2 API.
    ///
    /// Supports prefix filtering, delimiter-based grouping, and pagination
    /// via continuation tokens.
    pub async fn list_objects_v2(
        &self,
        request: ListObjectsV2Request,
    ) -> Result<ListObjectsV2Response> {
        let mut query: Vec<(&str, String)> = vec![("list-type", "2".to_string())];
        if let Some(ref prefix) = request.prefix {
            query.push(("prefix", prefix.clone()));
        }
        if let Some(ref delimiter) = request.delimiter {
            query.push(("delimiter", delimiter.clone()));
        }
        if let Some(max_keys) = request.max_keys {
            query.push(("max-keys", max_keys.to_string()));
        }
        if let Some(ref token) = request.continuation_token {
            query.push(("continuation-token", token.clone()));
        }
        if let Some(ref start_after) = request.start_after {
            query.push(("start-after", start_after.clone()));
        }

        let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let url = self.build_url(Some(&request.bucket), None, &query_refs)?;
        let http_req = self.http_client().request(Method::GET, url).build()?;
        let response = self.execute(http_req).await?;

        let body = response.text().await?;
        let list_resp: ListObjectsV2Response = parse_xml(&body)?;

        Ok(list_resp)
    }

    /// Copy an object within OSS.
    ///
    /// The source is specified via `x-oss-copy-source` header with the format
    /// `/{source_bucket}/{source_key}` (percent-encoded).
    pub async fn copy_object(&self, request: CopyObjectRequest) -> Result<CopyObjectResponse> {
        let url = self.build_url(Some(&request.bucket), Some(&request.key), &[])?;
        let mut http_req = self.http_client().request(Method::PUT, url);

        let encoded_key =
            utf8_percent_encode(request.source_key.as_ref(), URI_ENCODE_SET).to_string();
        let copy_source = format!("/{}/{}", request.source_bucket, encoded_key);
        http_req = http_req.header("x-oss-copy-source", &copy_source);

        if let Some(directive) = request.metadata_directive {
            http_req = http_req.header("x-oss-metadata-directive", directive.to_string());
        }
        if let Some(ref ct) = request.content_type {
            http_req = http_req.header("content-type", ct.as_str());
        }
        if let Some(sc) = request.storage_class {
            http_req = http_req.header("x-oss-storage-class", sc.to_string());
        }
        if let Some(acl) = request.acl {
            http_req = http_req.header("x-oss-object-acl", acl.to_string());
        }
        for (k, v) in &request.metadata {
            http_req = http_req.header(format!("x-oss-meta-{k}"), v.as_str());
        }

        let http_req = http_req.build()?;
        let response = self.execute(http_req).await?;

        let body = response.text().await?;
        let copy_resp: CopyObjectResponse = parse_xml(&body)?;

        Ok(copy_resp)
    }

    /// Delete multiple objects from OSS in a single request.
    ///
    /// Supports deleting up to 1000 objects per request. Uses quiet mode by default,
    /// which only returns errors (not successful deletions).
    pub async fn delete_multiple_objects(
        &self,
        request: DeleteMultipleObjectsRequest,
    ) -> Result<DeleteMultipleObjectsResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[("delete", "")])?;

        let xml_body = DeleteMultipleObjectsXml {
            quiet: request.quiet,
            objects: request
                .keys
                .iter()
                .map(|k| DeleteObjectXmlEntry {
                    key: k.as_ref().to_string(),
                })
                .collect(),
        };
        let body_str = serialize_xml(&xml_body)?;

        let digest = Md5::digest(body_str.as_bytes());
        let content_md5 = base64::engine::general_purpose::STANDARD.encode(digest.as_slice());

        let http_req = self
            .http_client()
            .request(Method::POST, url)
            .header("content-type", "application/xml")
            .header("content-md5", &content_md5)
            .body(body_str)
            .build()?;
        let response = self.execute(http_req).await?;

        let body = response.text().await?;
        if body.is_empty() {
            return Ok(DeleteMultipleObjectsResponse {
                deleted: Vec::new(),
            });
        }
        let delete_resp: DeleteMultipleObjectsResponse = parse_xml(&body)?;

        Ok(delete_resp)
    }

    /// Restore an archived object so it can be downloaded.
    ///
    /// The `days` parameter specifies how many days the restored copy remains available.
    pub async fn restore_object(
        &self,
        request: RestoreObjectRequest,
    ) -> Result<RestoreObjectResponse> {
        let url = self.build_url(
            Some(&request.bucket),
            Some(&request.key),
            &[("restore", "")],
        )?;
        let xml_body = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
             <RestoreRequest>\
             <Days>{}</Days>\
             </RestoreRequest>",
            request.days
        );
        let http_req = self
            .http_client()
            .request(Method::POST, url)
            .header("content-type", "application/xml")
            .body(xml_body)
            .build()?;
        let response = self.execute(http_req).await?;
        let request_id = header_opt(&response, "x-oss-request-id");
        Ok(RestoreObjectResponse { request_id })
    }

    /// Append data to an appendable object.
    ///
    /// Use `position: 0` when creating a new appendable object, or
    /// use the `next_append_position` from the previous response for subsequent appends.
    ///
    /// # Payload Signing
    ///
    /// When the body is backed by in-memory bytes (e.g., `Vec<u8>`, `Bytes`),
    /// the SDK computes a SHA-256 hash of the payload and includes it in the
    /// V4 signature. When the body is a non-buffered stream, the SDK uses
    /// `UNSIGNED-PAYLOAD` — the request is still authenticated via the
    /// Authorization header, but the payload itself is not integrity-checked
    /// by the signature. OSS may still validate Content-MD5 or CRC64 if
    /// those headers are present.
    pub async fn append_object(
        &self,
        request: AppendObjectRequest,
    ) -> Result<AppendObjectResponse> {
        let position_str = request.position.to_string();
        let url = self.build_url(
            Some(&request.bucket),
            Some(&request.key),
            &[("append", ""), ("position", &position_str)],
        )?;
        let mut http_req = self.http_client().request(Method::POST, url);
        if let Some(ref ct) = request.content_type {
            http_req = http_req.header("content-type", ct.as_str());
        }
        let http_req = http_req.body(request.body).build()?;
        let response = self.execute(http_req).await?;

        let next_append_position = header_opt(&response, "x-oss-next-append-position")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        let crc64 = header_opt(&response, "x-oss-hash-crc64ecma");
        let request_id = header_opt(&response, "x-oss-request-id");

        Ok(AppendObjectResponse {
            next_append_position,
            crc64,
            request_id,
        })
    }

    /// Get the ACL of an object.
    pub async fn get_object_acl(
        &self,
        request: GetObjectAclRequest,
    ) -> Result<GetObjectAclResponse> {
        let url = self.build_url(Some(&request.bucket), Some(&request.key), &[("acl", "")])?;
        let http_req = self.http_client().request(Method::GET, url).build()?;
        let response = self.execute(http_req).await?;
        let body = response.text().await?;
        let resp: GetObjectAclResponse = parse_xml(&body)?;
        Ok(resp)
    }

    /// Set the ACL of an object.
    pub async fn put_object_acl(
        &self,
        request: PutObjectAclRequest,
    ) -> Result<PutObjectAclResponse> {
        let url = self.build_url(Some(&request.bucket), Some(&request.key), &[("acl", "")])?;
        let http_req = self
            .http_client()
            .request(Method::PUT, url)
            .header("x-oss-object-acl", request.acl.to_string())
            .build()?;
        let response = self.execute(http_req).await?;
        let request_id = header_opt(&response, "x-oss-request-id");
        Ok(PutObjectAclResponse { request_id })
    }

    /// Get the tags of an object.
    pub async fn get_object_tagging(
        &self,
        request: GetObjectTaggingRequest,
    ) -> Result<GetObjectTaggingResponse> {
        let url = self.build_url(
            Some(&request.bucket),
            Some(&request.key),
            &[("tagging", "")],
        )?;
        let http_req = self.http_client().request(Method::GET, url).build()?;
        let response = self.execute(http_req).await?;
        let body = response.text().await?;
        let resp: GetObjectTaggingResponse = parse_xml(&body)?;
        Ok(resp)
    }

    /// Set the tags of an object (replaces all existing tags).
    pub async fn put_object_tagging(
        &self,
        request: PutObjectTaggingRequest,
    ) -> Result<PutObjectTaggingResponse> {
        let url = self.build_url(
            Some(&request.bucket),
            Some(&request.key),
            &[("tagging", "")],
        )?;
        let tag_set = TagSet {
            tags: request
                .tags
                .into_iter()
                .map(|(k, v)| Tag { key: k, value: v })
                .collect(),
        };
        let wrapper = TaggingXml { tag_set };
        let body_str = serialize_xml(&wrapper)?;

        let http_req = self
            .http_client()
            .request(Method::PUT, url)
            .header("content-type", "application/xml")
            .body(body_str)
            .build()?;
        let response = self.execute(http_req).await?;
        let request_id = header_opt(&response, "x-oss-request-id");
        Ok(PutObjectTaggingResponse { request_id })
    }

    /// Delete all tags from an object.
    pub async fn delete_object_tagging(
        &self,
        request: DeleteObjectTaggingRequest,
    ) -> Result<DeleteObjectTaggingResponse> {
        let url = self.build_url(
            Some(&request.bucket),
            Some(&request.key),
            &[("tagging", "")],
        )?;
        let http_req = self.http_client().request(Method::DELETE, url).build()?;
        let response = self.execute(http_req).await?;
        let request_id = header_opt(&response, "x-oss-request-id");
        Ok(DeleteObjectTaggingResponse { request_id })
    }
}
