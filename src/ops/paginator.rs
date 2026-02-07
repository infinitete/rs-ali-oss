//! Auto-paginators for listing operations.

use crate::client::OssClient;
use crate::error::Result;
use crate::types::common::BucketName;
use crate::types::response::{BucketInfo, ListBucketsResponse, ListObjectsV2Response, ObjectInfo};

/// A paginator that yields [`ObjectInfo`] items across all pages of a ListObjectsV2 call.
pub struct ListObjectsV2Paginator {
    client: OssClient,
    bucket: BucketName,
    prefix: Option<String>,
    delimiter: Option<String>,
    max_keys: Option<u32>,
    start_after: Option<String>,
    continuation_token: Option<String>,
    buffer: std::collections::VecDeque<ObjectInfo>,
    done: bool,
}

impl ListObjectsV2Paginator {
    pub(crate) fn new(
        client: OssClient,
        bucket: BucketName,
        prefix: Option<String>,
        delimiter: Option<String>,
        max_keys: Option<u32>,
        start_after: Option<String>,
    ) -> Self {
        Self {
            client,
            bucket,
            prefix,
            delimiter,
            max_keys,
            start_after,
            continuation_token: None,
            buffer: std::collections::VecDeque::new(),
            done: false,
        }
    }

    async fn fetch_next_page(&mut self) -> Result<()> {
        use crate::types::request::ListObjectsV2RequestBuilder;

        let mut builder = ListObjectsV2RequestBuilder::new().bucket(self.bucket.clone());

        if let Some(ref prefix) = self.prefix {
            builder = builder.prefix(prefix.clone());
        }
        if let Some(ref delimiter) = self.delimiter {
            builder = builder.delimiter(delimiter.clone());
        }
        if let Some(max_keys) = self.max_keys {
            builder = builder.max_keys(max_keys);
        }
        if let Some(ref start_after) = self.start_after {
            builder = builder.start_after(start_after.clone());
        }
        if let Some(ref token) = self.continuation_token {
            builder = builder.continuation_token(token.clone());
        }

        let request = builder.build()?;
        let response = self.client.list_objects_v2(request).await?;

        self.buffer.extend(response.contents);

        if response.is_truncated {
            self.continuation_token = response.next_continuation_token;
        } else {
            self.done = true;
        }

        Ok(())
    }

    /// Collect all objects across all pages into a single Vec.
    pub async fn collect_all(mut self) -> Result<Vec<ObjectInfo>> {
        let mut all = Vec::new();
        while !self.done {
            self.fetch_next_page().await?;
            all.extend(self.buffer.drain(..));
        }
        Ok(all)
    }

    /// Get the raw next page response (useful when you need metadata like common_prefixes).
    pub async fn next_page(&mut self) -> Result<Option<ListObjectsV2Response>> {
        if self.done {
            return Ok(None);
        }

        use crate::types::request::ListObjectsV2RequestBuilder;

        let mut builder = ListObjectsV2RequestBuilder::new().bucket(self.bucket.clone());
        if let Some(ref prefix) = self.prefix {
            builder = builder.prefix(prefix.clone());
        }
        if let Some(ref delimiter) = self.delimiter {
            builder = builder.delimiter(delimiter.clone());
        }
        if let Some(max_keys) = self.max_keys {
            builder = builder.max_keys(max_keys);
        }
        if let Some(ref start_after) = self.start_after {
            builder = builder.start_after(start_after.clone());
        }
        if let Some(ref token) = self.continuation_token {
            builder = builder.continuation_token(token.clone());
        }

        let request = builder.build()?;
        let response = self.client.list_objects_v2(request).await?;

        if response.is_truncated {
            self.continuation_token = response.next_continuation_token.clone();
        } else {
            self.done = true;
        }

        Ok(Some(response))
    }
}

/// A paginator that yields [`BucketInfo`] items across all pages of a ListBuckets call.
pub struct ListBucketsPaginator {
    client: OssClient,
    prefix: Option<String>,
    max_keys: Option<u32>,
    marker: Option<String>,
    buffer: std::collections::VecDeque<BucketInfo>,
    done: bool,
}

impl ListBucketsPaginator {
    pub(crate) fn new(client: OssClient, prefix: Option<String>, max_keys: Option<u32>) -> Self {
        Self {
            client,
            prefix,
            max_keys,
            marker: None,
            buffer: std::collections::VecDeque::new(),
            done: false,
        }
    }

    async fn fetch_next_page(&mut self) -> Result<()> {
        use crate::types::request::ListBucketsRequestBuilder;

        let mut builder = ListBucketsRequestBuilder::new();
        if let Some(ref prefix) = self.prefix {
            builder = builder.prefix(prefix.clone());
        }
        if let Some(max_keys) = self.max_keys {
            builder = builder.max_keys(max_keys);
        }
        if let Some(ref marker) = self.marker {
            builder = builder.marker(marker.clone());
        }

        let request = builder.build()?;
        let response = self.client.list_buckets(request).await?;

        self.buffer.extend(response.buckets.bucket);

        if response.is_truncated {
            self.marker = response.next_marker;
        } else {
            self.done = true;
        }

        Ok(())
    }

    /// Collect all buckets across all pages into a single Vec.
    pub async fn collect_all(mut self) -> Result<Vec<BucketInfo>> {
        let mut all = Vec::new();
        while !self.done {
            self.fetch_next_page().await?;
            all.extend(self.buffer.drain(..));
        }
        Ok(all)
    }

    /// Get the raw next page response.
    pub async fn next_page(&mut self) -> Result<Option<ListBucketsResponse>> {
        if self.done {
            return Ok(None);
        }

        use crate::types::request::ListBucketsRequestBuilder;

        let mut builder = ListBucketsRequestBuilder::new();
        if let Some(ref prefix) = self.prefix {
            builder = builder.prefix(prefix.clone());
        }
        if let Some(max_keys) = self.max_keys {
            builder = builder.max_keys(max_keys);
        }
        if let Some(ref marker) = self.marker {
            builder = builder.marker(marker.clone());
        }

        let request = builder.build()?;
        let response = self.client.list_buckets(request).await?;

        if response.is_truncated {
            self.marker = response.next_marker.clone();
        } else {
            self.done = true;
        }

        Ok(Some(response))
    }
}

impl OssClient {
    /// Create a paginator that auto-fetches all pages of list_objects_v2.
    pub fn list_objects_v2_paginator(&self, bucket: BucketName) -> ListObjectsV2PaginatorBuilder {
        ListObjectsV2PaginatorBuilder {
            client: self.clone(),
            bucket,
            prefix: None,
            delimiter: None,
            max_keys: None,
            start_after: None,
        }
    }

    /// Create a paginator that auto-fetches all pages of list_buckets.
    pub fn list_buckets_paginator(&self) -> ListBucketsPaginatorBuilder {
        ListBucketsPaginatorBuilder {
            client: self.clone(),
            prefix: None,
            max_keys: None,
        }
    }
}

/// Builder for [`ListObjectsV2Paginator`].
pub struct ListObjectsV2PaginatorBuilder {
    client: OssClient,
    bucket: BucketName,
    prefix: Option<String>,
    delimiter: Option<String>,
    max_keys: Option<u32>,
    start_after: Option<String>,
}

impl ListObjectsV2PaginatorBuilder {
    /// Filter results to keys beginning with this prefix.
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Group keys by this delimiter.
    pub fn delimiter(mut self, delimiter: impl Into<String>) -> Self {
        self.delimiter = Some(delimiter.into());
        self
    }

    /// Maximum keys per page (1-1000).
    pub fn max_keys(mut self, max_keys: u32) -> Self {
        self.max_keys = Some(max_keys);
        self
    }

    /// Start listing after this key.
    pub fn start_after(mut self, start_after: impl Into<String>) -> Self {
        self.start_after = Some(start_after.into());
        self
    }

    /// Build the paginator.
    pub fn build(self) -> ListObjectsV2Paginator {
        ListObjectsV2Paginator::new(
            self.client,
            self.bucket,
            self.prefix,
            self.delimiter,
            self.max_keys,
            self.start_after,
        )
    }
}

/// Builder for [`ListBucketsPaginator`].
pub struct ListBucketsPaginatorBuilder {
    client: OssClient,
    prefix: Option<String>,
    max_keys: Option<u32>,
}

impl ListBucketsPaginatorBuilder {
    /// Filter buckets by name prefix.
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Maximum buckets per page.
    pub fn max_keys(mut self, max_keys: u32) -> Self {
        self.max_keys = Some(max_keys);
        self
    }

    /// Build the paginator.
    pub fn build(self) -> ListBucketsPaginator {
        ListBucketsPaginator::new(self.client, self.prefix, self.max_keys)
    }
}
