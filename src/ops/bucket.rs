//! Bucket operations: CreateBucket, DeleteBucket, ListBuckets, GetBucketInfo.

use reqwest::Method;

use crate::client::{OssClient, header_opt, parse_xml};
use crate::error::Result;
use crate::types::request::{
    CreateBucketRequest, DeleteBucketRequest, GetBucketInfoRequest, GetBucketLocationRequest,
    ListBucketsRequest,
};
use crate::types::response::{
    CreateBucketResponse, DeleteBucketResponse, GetBucketInfoResponse, GetBucketLocationResponse,
    ListBucketsResponse,
};

impl OssClient {
    /// Create a new bucket.
    ///
    /// Optionally specify a storage class; defaults to Standard if omitted.
    pub async fn create_bucket(
        &self,
        request: CreateBucketRequest,
    ) -> Result<CreateBucketResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[])?;
        let mut http_req = self.http_client().request(Method::PUT, url);

        let body = match request.storage_class {
            Some(sc) => {
                let xml = format!(
                    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
                     <CreateBucketConfiguration>\
                     <StorageClass>{sc}</StorageClass>\
                     </CreateBucketConfiguration>"
                );
                http_req = http_req.header("content-type", "application/xml");
                Some(xml)
            }
            None => None,
        };

        let http_req = if let Some(xml_body) = body {
            http_req.body(xml_body).build()?
        } else {
            http_req.build()?
        };

        let response = self.execute(http_req).await?;

        let request_id = header_opt(&response, "x-oss-request-id");

        Ok(CreateBucketResponse { request_id })
    }

    /// Delete a bucket.
    ///
    /// The bucket must be empty before it can be deleted.
    pub async fn delete_bucket(
        &self,
        request: DeleteBucketRequest,
    ) -> Result<DeleteBucketResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[])?;
        let http_req = self.http_client().request(Method::DELETE, url).build()?;
        let response = self.execute(http_req).await?;

        let request_id = header_opt(&response, "x-oss-request-id");

        Ok(DeleteBucketResponse { request_id })
    }

    /// List all buckets owned by the authenticated user.
    ///
    /// This operation targets the region endpoint without a bucket in the host.
    pub async fn list_buckets(&self, request: ListBucketsRequest) -> Result<ListBucketsResponse> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(ref prefix) = request.prefix {
            query.push(("prefix", prefix.clone()));
        }
        if let Some(ref marker) = request.marker {
            query.push(("marker", marker.clone()));
        }
        if let Some(max_keys) = request.max_keys {
            query.push(("max-keys", max_keys.to_string()));
        }

        let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let url = self.build_url(None, None, &query_refs)?;
        let http_req = self.http_client().request(Method::GET, url).build()?;
        let response = self.execute(http_req).await?;

        let body = response.text().await?;
        let list_resp: ListBucketsResponse = parse_xml(&body)?;

        Ok(list_resp)
    }

    /// Retrieve bucket metadata and configuration.
    pub async fn get_bucket_info(
        &self,
        request: GetBucketInfoRequest,
    ) -> Result<GetBucketInfoResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[("bucketInfo", "")])?;
        let http_req = self.http_client().request(Method::GET, url).build()?;
        let response = self.execute(http_req).await?;

        let body = response.text().await?;
        let info_resp: GetBucketInfoResponse = parse_xml(&body)?;

        Ok(info_resp)
    }

    /// Get the region/location of a bucket.
    pub async fn get_bucket_location(
        &self,
        request: GetBucketLocationRequest,
    ) -> Result<GetBucketLocationResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[("location", "")])?;
        let http_req = self.http_client().request(Method::GET, url).build()?;
        let response = self.execute(http_req).await?;

        let body = response.text().await?;
        let xml: crate::types::response::LocationConstraintXml = parse_xml(&body)?;

        Ok(GetBucketLocationResponse {
            location: xml.location,
        })
    }
}
