//! Multipart upload operations: Initiate, UploadPart, Complete, Abort, ListParts.

use reqwest::Method;

use crate::client::{OssClient, header_etag, header_opt, parse_xml, serialize_xml};
use crate::error::Result;
use crate::types::request::{
    AbortMultipartUploadRequest, CompleteMultipartUploadRequest, CompleteMultipartUploadXml,
    InitiateMultipartUploadRequest, ListMultipartUploadsRequest, ListPartsRequest,
    UploadPartRequest,
};
use crate::types::response::{
    AbortMultipartUploadResponse, CompleteMultipartUploadResponse, InitiateMultipartUploadResponse,
    ListMultipartUploadsResponse, ListPartsResponse, UploadPartResponse,
};

impl OssClient {
    /// Initiate a multipart upload and obtain an upload ID.
    pub async fn initiate_multipart_upload(
        &self,
        request: InitiateMultipartUploadRequest,
    ) -> Result<InitiateMultipartUploadResponse> {
        let url = self.build_url(
            Some(&request.bucket),
            Some(&request.key),
            &[("uploads", "")],
        )?;
        let mut http_req = self.http_client().request(Method::POST, url);

        if let Some(ref ct) = request.content_type {
            http_req = http_req.header("content-type", ct.as_str());
        }
        if let Some(sc) = request.storage_class {
            http_req = http_req.header("x-oss-storage-class", sc.to_string());
        }

        let http_req = http_req.build()?;
        let response = self.execute(http_req).await?;

        let body = response.text().await?;
        let init_resp: InitiateMultipartUploadResponse = parse_xml(&body)?;

        Ok(init_resp)
    }

    /// Upload a single part of a multipart upload.
    ///
    /// # Payload Signing
    ///
    /// When the body is in-memory bytes the SDK computes a SHA-256 payload
    /// hash for the V4 signature. For non-buffered streaming bodies the SDK
    /// uses `UNSIGNED-PAYLOAD` — the request is authenticated but the payload
    /// is not integrity-checked by the signature.
    pub async fn upload_part(&self, request: UploadPartRequest) -> Result<UploadPartResponse> {
        let part_num = request.part_number.to_string();
        let query = [
            ("partNumber", part_num.as_str()),
            ("uploadId", request.upload_id.as_str()),
        ];
        let url = self.build_url(Some(&request.bucket), Some(&request.key), &query)?;
        let http_req = self
            .http_client()
            .request(Method::PUT, url)
            .body(request.body)
            .build()?;
        let response = self.execute(http_req).await?;

        let etag = header_etag(&response);

        Ok(UploadPartResponse { etag })
    }

    /// Complete a multipart upload by assembling previously uploaded parts.
    pub async fn complete_multipart_upload(
        &self,
        request: CompleteMultipartUploadRequest,
    ) -> Result<CompleteMultipartUploadResponse> {
        let query = [("uploadId", request.upload_id.as_str())];
        let url = self.build_url(Some(&request.bucket), Some(&request.key), &query)?;

        let xml_body = CompleteMultipartUploadXml {
            parts: request.parts,
        };
        let body_str = serialize_xml(&xml_body)?;

        let http_req = self
            .http_client()
            .request(Method::POST, url)
            .header("content-type", "application/xml")
            .body(body_str)
            .build()?;
        let response = self.execute(http_req).await?;

        let body = response.text().await?;
        let complete_resp: CompleteMultipartUploadResponse = parse_xml(&body)?;

        Ok(complete_resp)
    }

    /// Abort a multipart upload and discard all uploaded parts.
    pub async fn abort_multipart_upload(
        &self,
        request: AbortMultipartUploadRequest,
    ) -> Result<AbortMultipartUploadResponse> {
        let query = [("uploadId", request.upload_id.as_str())];
        let url = self.build_url(Some(&request.bucket), Some(&request.key), &query)?;
        let http_req = self.http_client().request(Method::DELETE, url).build()?;
        let response = self.execute(http_req).await?;

        let request_id = header_opt(&response, "x-oss-request-id");

        Ok(AbortMultipartUploadResponse { request_id })
    }

    /// List parts that have been uploaded for a multipart upload.
    pub async fn list_parts(&self, request: ListPartsRequest) -> Result<ListPartsResponse> {
        let mut query: Vec<(&str, String)> = vec![("uploadId", request.upload_id.clone())];
        if let Some(max_parts) = request.max_parts {
            query.push(("max-parts", max_parts.to_string()));
        }
        if let Some(marker) = request.part_number_marker {
            query.push(("part-number-marker", marker.to_string()));
        }

        let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let url = self.build_url(Some(&request.bucket), Some(&request.key), &query_refs)?;
        let http_req = self.http_client().request(Method::GET, url).build()?;
        let response = self.execute(http_req).await?;

        let body = response.text().await?;
        let list_resp: ListPartsResponse = parse_xml(&body)?;

        Ok(list_resp)
    }

    /// List in-progress multipart uploads for a bucket.
    pub async fn list_multipart_uploads(
        &self,
        request: ListMultipartUploadsRequest,
    ) -> Result<ListMultipartUploadsResponse> {
        let mut query: Vec<(&str, String)> = vec![("uploads", String::new())];
        if let Some(ref prefix) = request.prefix {
            query.push(("prefix", prefix.clone()));
        }
        if let Some(ref delimiter) = request.delimiter {
            query.push(("delimiter", delimiter.clone()));
        }
        if let Some(max_uploads) = request.max_uploads {
            query.push(("max-uploads", max_uploads.to_string()));
        }
        if let Some(ref key_marker) = request.key_marker {
            query.push(("key-marker", key_marker.clone()));
        }
        if let Some(ref upload_id_marker) = request.upload_id_marker {
            query.push(("upload-id-marker", upload_id_marker.clone()));
        }

        let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let url = self.build_url(Some(&request.bucket), None, &query_refs)?;
        let http_req = self.http_client().request(Method::GET, url).build()?;
        let response = self.execute(http_req).await?;

        let body = response.text().await?;
        let resp: ListMultipartUploadsResponse = parse_xml(&body)?;

        Ok(resp)
    }
}
