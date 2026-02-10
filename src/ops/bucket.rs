//! Bucket operations: CreateBucket, DeleteBucket, ListBuckets, GetBucketInfo, BucketAcl, BucketCors, BucketReferer, BucketPolicy, BucketVersioning, BucketLifecycle, BucketEncryption, BucketLogging.

use reqwest::Method;

use crate::client::{OssClient, header_opt, parse_xml, serialize_xml};
use crate::error::Result;
use crate::types::request::{
    CorsConfigurationXml, CorsRuleXml, CreateBucketRequest, DeleteBucketCorsRequest,
    DeleteBucketEncryptionRequest, DeleteBucketLifecycleRequest, DeleteBucketLoggingRequest,
    DeleteBucketPolicyRequest, DeleteBucketRequest, EncryptionConfigurationXml, EncryptionRuleXml,
    GetBucketAclRequest, GetBucketCorsRequest, GetBucketEncryptionRequest, GetBucketInfoRequest,
    GetBucketLifecycleRequest, GetBucketLocationRequest, GetBucketLoggingRequest,
    GetBucketPolicyRequest, GetBucketRefererRequest, GetBucketVersioningRequest,
    LifecycleConfigurationXml, LifecycleExpirationXml, LifecycleRuleXml, LifecycleTransitionXml,
    ListBucketsRequest, LoggingConfigurationXml, LoggingEnabledXml, PutBucketAclRequest,
    PutBucketCorsRequest, PutBucketEncryptionRequest, PutBucketLifecycleRequest,
    PutBucketLoggingRequest, PutBucketPolicyRequest, PutBucketRefererRequest,
    PutBucketVersioningRequest, RefererBlacklistXml, RefererConfigurationXml, RefererListXml,
    VersioningConfigurationXml,
};
use crate::types::response::{
    CreateBucketResponse, DeleteBucketCorsResponse, DeleteBucketEncryptionResponse,
    DeleteBucketLifecycleResponse, DeleteBucketLoggingResponse, DeleteBucketPolicyResponse,
    DeleteBucketResponse, GetBucketAclResponse, GetBucketCorsResponse, GetBucketEncryptionResponse,
    GetBucketInfoResponse, GetBucketLifecycleResponse, GetBucketLocationResponse,
    GetBucketLoggingResponse, GetBucketPolicyResponse, GetBucketRefererResponse,
    GetBucketVersioningResponse, ListBucketsResponse, PutBucketAclResponse, PutBucketCorsResponse,
    PutBucketEncryptionResponse, PutBucketLifecycleResponse, PutBucketLoggingResponse,
    PutBucketPolicyResponse, PutBucketRefererResponse, PutBucketVersioningResponse,
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
        let resource_path = format!("/{}/", request.bucket);
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

        let response = self.execute(http_req, &resource_path).await?;

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
        let resource_path = format!("/{}/", request.bucket);
        let http_req = self.http_client().request(Method::DELETE, url).build()?;
        let response = self.execute(http_req, &resource_path).await?;

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
        let response = self.execute(http_req, "/").await?;

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
        let resource_path = format!("/{}/", request.bucket);
        let http_req = self.http_client().request(Method::GET, url).build()?;
        let response = self.execute(http_req, &resource_path).await?;

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
        let resource_path = format!("/{}/", request.bucket);
        let http_req = self.http_client().request(Method::GET, url).build()?;
        let response = self.execute(http_req, &resource_path).await?;

        let body = response.text().await?;
        let xml: crate::types::response::LocationConstraintXml = parse_xml(&body)?;

        Ok(GetBucketLocationResponse {
            location: xml.location,
        })
    }

    /// Set the ACL of a bucket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rs_ali_oss::*;
    /// # use rs_ali_oss::types::request::PutBucketAclRequestBuilder;
    /// # async fn example(client: OssClient) -> Result<()> {
    /// let request = PutBucketAclRequestBuilder::new()
    ///     .bucket(BucketName::new("my-bucket")?)
    ///     .acl(BucketAcl::PublicRead)
    ///     .build()?;
    /// client.put_bucket_acl(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn put_bucket_acl(
        &self,
        request: PutBucketAclRequest,
    ) -> Result<PutBucketAclResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[("acl", "")])?;
        let resource_path = format!("/{}/", request.bucket);
        let http_req = self
            .http_client()
            .request(Method::PUT, url)
            .header("x-oss-acl", request.acl.to_string())
            .build()?;
        let response = self.execute(http_req, &resource_path).await?;
        let request_id = header_opt(&response, "x-oss-request-id");
        Ok(PutBucketAclResponse { request_id })
    }

    /// Get the ACL of a bucket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rs_ali_oss::*;
    /// # use rs_ali_oss::types::request::GetBucketAclRequestBuilder;
    /// # async fn example(client: OssClient) -> Result<()> {
    /// let request = GetBucketAclRequestBuilder::new()
    ///     .bucket(BucketName::new("my-bucket")?)
    ///     .build()?;
    /// let response = client.get_bucket_acl(request).await?;
    /// println!("ACL: {}", response.access_control_list.grant);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_bucket_acl(
        &self,
        request: GetBucketAclRequest,
    ) -> Result<GetBucketAclResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[("acl", "")])?;
        let resource_path = format!("/{}/", request.bucket);
        let http_req = self.http_client().request(Method::GET, url).build()?;
        let response = self.execute(http_req, &resource_path).await?;
        let body = response.text().await?;
        let resp: GetBucketAclResponse = parse_xml(&body)?;
        Ok(resp)
    }

    /// Set the CORS configuration of a bucket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rs_ali_oss::*;
    /// # use rs_ali_oss::types::request::{PutBucketCorsRequestBuilder, CorsRule};
    /// # use rs_ali_oss::types::common::CorsHttpMethod;
    /// # async fn example(client: OssClient) -> Result<()> {
    /// let rule = CorsRule::new()
    ///     .add_allowed_origin("*")
    ///     .add_allowed_method(CorsHttpMethod::Get)
    ///     .add_allowed_method(CorsHttpMethod::Put)
    ///     .allowed_headers(vec!["*".to_string()])
    ///     .max_age_seconds(100);
    ///
    /// let request = PutBucketCorsRequestBuilder::new()
    ///     .bucket(BucketName::new("my-bucket")?)
    ///     .add_rule(rule)
    ///     .build()?;
    /// client.put_bucket_cors(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn put_bucket_cors(
        &self,
        request: PutBucketCorsRequest,
    ) -> Result<PutBucketCorsResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[("cors", "")])?;
        let resource_path = format!("/{}/", request.bucket);

        // Convert CorsRule to CorsRuleXml
        let cors_rules_xml: Vec<CorsRuleXml> = request
            .cors_rules
            .into_iter()
            .map(|rule| CorsRuleXml {
                allowed_origins: rule.allowed_origins,
                allowed_methods: rule
                    .allowed_methods
                    .into_iter()
                    .map(|m| m.to_string())
                    .collect(),
                allowed_headers: rule.allowed_headers.unwrap_or_default(),
                expose_headers: rule.expose_headers.unwrap_or_default(),
                max_age_seconds: rule.max_age_seconds,
            })
            .collect();

        let config = CorsConfigurationXml {
            cors_rules: cors_rules_xml,
            response_vary: None,
        };

        let xml_body = serialize_xml(&config)?;
        let mut http_req = self
            .http_client()
            .request(Method::PUT, url)
            .header("content-type", "application/xml");
        http_req = http_req.body(xml_body);
        let http_req = http_req.build()?;

        let response = self.execute(http_req, &resource_path).await?;
        let request_id = header_opt(&response, "x-oss-request-id");
        Ok(PutBucketCorsResponse { request_id })
    }

    /// Get the CORS configuration of a bucket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rs_ali_oss::*;
    /// # use rs_ali_oss::types::request::GetBucketCorsRequestBuilder;
    /// # async fn example(client: OssClient) -> Result<()> {
    /// let request = GetBucketCorsRequestBuilder::new()
    ///     .bucket(BucketName::new("my-bucket")?)
    ///     .build()?;
    /// let response = client.get_bucket_cors(request).await?;
    /// println!("CORS rules: {}", response.cors_rules.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_bucket_cors(
        &self,
        request: GetBucketCorsRequest,
    ) -> Result<GetBucketCorsResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[("cors", "")])?;
        let resource_path = format!("/{}/", request.bucket);
        let http_req = self.http_client().request(Method::GET, url).build()?;
        let response = self.execute(http_req, &resource_path).await?;
        let body = response.text().await?;
        let resp: GetBucketCorsResponse = parse_xml(&body)?;
        Ok(resp)
    }

    /// Delete the CORS configuration of a bucket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rs_ali_oss::*;
    /// # use rs_ali_oss::types::request::DeleteBucketCorsRequestBuilder;
    /// # async fn example(client: OssClient) -> Result<()> {
    /// let request = DeleteBucketCorsRequestBuilder::new()
    ///     .bucket(BucketName::new("my-bucket")?)
    ///     .build()?;
    /// client.delete_bucket_cors(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_bucket_cors(
        &self,
        request: DeleteBucketCorsRequest,
    ) -> Result<DeleteBucketCorsResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[("cors", "")])?;
        let resource_path = format!("/{}/", request.bucket);
        let http_req = self.http_client().request(Method::DELETE, url).build()?;
        let response = self.execute(http_req, &resource_path).await?;
        let request_id = header_opt(&response, "x-oss-request-id");
        Ok(DeleteBucketCorsResponse { request_id })
    }

    /// Set the Referer (hotlink protection) configuration of a bucket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rs_ali_oss::*;
    /// # use rs_ali_oss::types::request::PutBucketRefererRequestBuilder;
    /// # async fn example(client: OssClient) -> Result<()> {
    /// let request = PutBucketRefererRequestBuilder::new()
    ///     .bucket(BucketName::new("my-bucket")?)
    ///     .allow_empty_referer(false)
    ///     .allow_truncate_query_string(true)
    ///     .add_referer("http://example.com")
    ///     .add_referer("https://example.com")
    ///     .referer_blacklist(vec!["http://refuse.com".to_string()])
    ///     .build()?;
    /// client.put_bucket_referer(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn put_bucket_referer(
        &self,
        request: PutBucketRefererRequest,
    ) -> Result<PutBucketRefererResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[("referer", "")])?;
        let resource_path = format!("/{}/", request.bucket);

        let referer_list = RefererListXml {
            referers: request.referer_list,
        };

        let referer_blacklist = request
            .referer_blacklist
            .map(|list| RefererBlacklistXml { referers: list });

        let config = RefererConfigurationXml {
            allow_empty_referer: request.allow_empty_referer,
            allow_truncate_query_string: request.allow_truncate_query_string,
            truncate_path: request.truncate_path,
            referer_list,
            referer_blacklist,
        };

        let xml_body = serialize_xml(&config)?;
        let mut http_req = self
            .http_client()
            .request(Method::PUT, url)
            .header("content-type", "application/xml");
        http_req = http_req.body(xml_body);
        let http_req = http_req.build()?;

        let response = self.execute(http_req, &resource_path).await?;
        let request_id = header_opt(&response, "x-oss-request-id");
        Ok(PutBucketRefererResponse { request_id })
    }

    /// Get the Referer configuration of a bucket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rs_ali_oss::*;
    /// # use rs_ali_oss::types::request::GetBucketRefererRequestBuilder;
    /// # async fn example(client: OssClient) -> Result<()> {
    /// let request = GetBucketRefererRequestBuilder::new()
    ///     .bucket(BucketName::new("my-bucket")?)
    ///     .build()?;
    /// let response = client.get_bucket_referer(request).await?;
    /// println!("Allow empty referer: {}", response.allow_empty_referer);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_bucket_referer(
        &self,
        request: GetBucketRefererRequest,
    ) -> Result<GetBucketRefererResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[("referer", "")])?;
        let resource_path = format!("/{}/", request.bucket);
        let http_req = self.http_client().request(Method::GET, url).build()?;
        let response = self.execute(http_req, &resource_path).await?;
        let body = response.text().await?;
        let resp: GetBucketRefererResponse = parse_xml(&body)?;
        Ok(resp)
    }

    /// Set the authorization policy of a bucket.
    ///
    /// The policy is a JSON string that defines permissions for the bucket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rs_ali_oss::*;
    /// # use rs_ali_oss::types::request::PutBucketPolicyRequestBuilder;
    /// # async fn example(client: OssClient) -> Result<()> {
    /// let policy = r#"{"Version":"1","Statement":[]}"#;
    /// let request = PutBucketPolicyRequestBuilder::new()
    ///     .bucket(BucketName::new("my-bucket")?)
    ///     .policy(policy)
    ///     .build()?;
    /// client.put_bucket_policy(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn put_bucket_policy(
        &self,
        request: PutBucketPolicyRequest,
    ) -> Result<PutBucketPolicyResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[("policy", "")])?;
        let resource_path = format!("/{}/", request.bucket);
        let mut http_req = self
            .http_client()
            .request(Method::PUT, url)
            .header("content-type", "application/json");
        http_req = http_req.body(request.policy);
        let http_req = http_req.build()?;
        let response = self.execute(http_req, &resource_path).await?;
        let request_id = header_opt(&response, "x-oss-request-id");
        Ok(PutBucketPolicyResponse { request_id })
    }

    /// Get the authorization policy of a bucket.
    ///
    /// Returns the policy as a JSON string.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rs_ali_oss::*;
    /// # use rs_ali_oss::types::request::GetBucketPolicyRequestBuilder;
    /// # async fn example(client: OssClient) -> Result<()> {
    /// let request = GetBucketPolicyRequestBuilder::new()
    ///     .bucket(BucketName::new("my-bucket")?)
    ///     .build()?;
    /// let response = client.get_bucket_policy(request).await?;
    /// println!("Policy: {}", response.policy);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_bucket_policy(
        &self,
        request: GetBucketPolicyRequest,
    ) -> Result<GetBucketPolicyResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[("policy", "")])?;
        let resource_path = format!("/{}/", request.bucket);
        let http_req = self.http_client().request(Method::GET, url).build()?;
        let response = self.execute(http_req, &resource_path).await?;
        let policy = response.text().await?;
        Ok(GetBucketPolicyResponse { policy })
    }

    /// Delete the authorization policy of a bucket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rs_ali_oss::*;
    /// # use rs_ali_oss::types::request::DeleteBucketPolicyRequestBuilder;
    /// # async fn example(client: OssClient) -> Result<()> {
    /// let request = DeleteBucketPolicyRequestBuilder::new()
    ///     .bucket(BucketName::new("my-bucket")?)
    ///     .build()?;
    /// client.delete_bucket_policy(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_bucket_policy(
        &self,
        request: DeleteBucketPolicyRequest,
    ) -> Result<DeleteBucketPolicyResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[("policy", "")])?;
        let resource_path = format!("/{}/", request.bucket);
        let http_req = self.http_client().request(Method::DELETE, url).build()?;
        let response = self.execute(http_req, &resource_path).await?;
        let request_id = header_opt(&response, "x-oss-request-id");
        Ok(DeleteBucketPolicyResponse { request_id })
    }

    /// Set the versioning status of a bucket.
    ///
    /// Once versioning is enabled, it can only be suspended, not fully disabled.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rs_ali_oss::*;
    /// # use rs_ali_oss::types::request::PutBucketVersioningRequestBuilder;
    /// # use rs_ali_oss::types::VersioningStatus;
    /// # async fn example(client: OssClient) -> Result<()> {
    /// let request = PutBucketVersioningRequestBuilder::new()
    ///     .bucket(BucketName::new("my-bucket")?)
    ///     .status(VersioningStatus::Enabled)
    ///     .build()?;
    /// client.put_bucket_versioning(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn put_bucket_versioning(
        &self,
        request: PutBucketVersioningRequest,
    ) -> Result<PutBucketVersioningResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[("versioning", "")])?;
        let resource_path = format!("/{}/", request.bucket);

        let config = VersioningConfigurationXml {
            status: request.status,
        };

        let xml_body = serialize_xml(&config)?;
        let mut http_req = self
            .http_client()
            .request(Method::PUT, url)
            .header("content-type", "application/xml");
        http_req = http_req.body(xml_body);
        let http_req = http_req.build()?;

        let response = self.execute(http_req, &resource_path).await?;
        let request_id = header_opt(&response, "x-oss-request-id");
        Ok(PutBucketVersioningResponse { request_id })
    }

    /// Get the versioning status of a bucket.
    ///
    /// Returns None if versioning has never been enabled.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rs_ali_oss::*;
    /// # use rs_ali_oss::types::request::GetBucketVersioningRequestBuilder;
    /// # async fn example(client: OssClient) -> Result<()> {
    /// let request = GetBucketVersioningRequestBuilder::new()
    ///     .bucket(BucketName::new("my-bucket")?)
    ///     .build()?;
    /// let response = client.get_bucket_versioning(request).await?;
    /// if let Some(status) = response.status {
    ///     println!("Versioning status: {}", status);
    /// } else {
    ///     println!("Versioning has never been enabled");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_bucket_versioning(
        &self,
        request: GetBucketVersioningRequest,
    ) -> Result<GetBucketVersioningResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[("versioning", "")])?;
        let resource_path = format!("/{}/", request.bucket);
        let http_req = self.http_client().request(Method::GET, url).build()?;
        let response = self.execute(http_req, &resource_path).await?;
        let body = response.text().await?;
        let resp: GetBucketVersioningResponse = parse_xml(&body)?;
        Ok(resp)
    }

    /// Set the lifecycle configuration of a bucket.
    ///
    /// Lifecycle rules define when objects should be expired or transitioned to different storage classes.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rs_ali_oss::*;
    /// # use rs_ali_oss::types::request::{PutBucketLifecycleRequestBuilder, LifecycleRule, LifecycleRuleStatus, LifecycleExpiration, LifecycleTransition};
    /// # async fn example(client: OssClient) -> Result<()> {
    /// let rule = LifecycleRule::new()
    ///     .id("delete-logs")
    ///     .prefix("logs/")
    ///     .status(LifecycleRuleStatus::Enabled)
    ///     .expiration(LifecycleExpiration::Days(30));
    ///
    /// let request = PutBucketLifecycleRequestBuilder::new()
    ///     .bucket(BucketName::new("my-bucket")?)
    ///     .add_rule(rule)
    ///     .build()?;
    /// client.put_bucket_lifecycle(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn put_bucket_lifecycle(
        &self,
        request: PutBucketLifecycleRequest,
    ) -> Result<PutBucketLifecycleResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[("lifecycle", "")])?;
        let resource_path = format!("/{}/", request.bucket);

        let rules_xml: Vec<LifecycleRuleXml> = request
            .lifecycle_rules
            .into_iter()
            .map(|rule| {
                let expiration_xml = rule.expiration.map(|exp| match exp {
                    crate::types::request::LifecycleExpiration::Days(days) => {
                        LifecycleExpirationXml {
                            days: Some(days),
                            date: None,
                        }
                    }
                    crate::types::request::LifecycleExpiration::Date(date) => {
                        LifecycleExpirationXml {
                            days: None,
                            date: Some(date),
                        }
                    }
                });

                let transitions_xml: Vec<LifecycleTransitionXml> = rule
                    .transitions
                    .into_iter()
                    .map(|trans| LifecycleTransitionXml {
                        days: trans.days,
                        storage_class: trans.storage_class.to_string(),
                    })
                    .collect();

                LifecycleRuleXml {
                    id: rule.id,
                    prefix: rule.prefix,
                    status: rule.status.as_str().to_string(),
                    expiration: expiration_xml,
                    transitions: transitions_xml,
                }
            })
            .collect();

        let config = LifecycleConfigurationXml { rules: rules_xml };
        let xml_body = serialize_xml(&config)?;
        let mut http_req = self
            .http_client()
            .request(Method::PUT, url)
            .header("content-type", "application/xml");
        http_req = http_req.body(xml_body);
        let http_req = http_req.build()?;

        let response = self.execute(http_req, &resource_path).await?;
        let request_id = header_opt(&response, "x-oss-request-id");
        Ok(PutBucketLifecycleResponse { request_id })
    }

    /// Get the lifecycle configuration of a bucket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rs_ali_oss::*;
    /// # use rs_ali_oss::types::request::GetBucketLifecycleRequestBuilder;
    /// # async fn example(client: OssClient) -> Result<()> {
    /// let request = GetBucketLifecycleRequestBuilder::new()
    ///     .bucket(BucketName::new("my-bucket")?)
    ///     .build()?;
    /// let response = client.get_bucket_lifecycle(request).await?;
    /// println!("Lifecycle rules: {}", response.rules.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_bucket_lifecycle(
        &self,
        request: GetBucketLifecycleRequest,
    ) -> Result<GetBucketLifecycleResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[("lifecycle", "")])?;
        let resource_path = format!("/{}/", request.bucket);
        let http_req = self.http_client().request(Method::GET, url).build()?;
        let response = self.execute(http_req, &resource_path).await?;
        let body = response.text().await?;
        let resp: GetBucketLifecycleResponse = parse_xml(&body)?;
        Ok(resp)
    }

    /// Delete the lifecycle configuration of a bucket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rs_ali_oss::*;
    /// # use rs_ali_oss::types::request::DeleteBucketLifecycleRequestBuilder;
    /// # async fn example(client: OssClient) -> Result<()> {
    /// let request = DeleteBucketLifecycleRequestBuilder::new()
    ///     .bucket(BucketName::new("my-bucket")?)
    ///     .build()?;
    /// client.delete_bucket_lifecycle(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_bucket_lifecycle(
        &self,
        request: DeleteBucketLifecycleRequest,
    ) -> Result<DeleteBucketLifecycleResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[("lifecycle", "")])?;
        let resource_path = format!("/{}/", request.bucket);
        let http_req = self.http_client().request(Method::DELETE, url).build()?;
        let response = self.execute(http_req, &resource_path).await?;
        let request_id = header_opt(&response, "x-oss-request-id");
        Ok(DeleteBucketLifecycleResponse { request_id })
    }

    /// Set the encryption configuration of a bucket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rs_ali_oss::*;
    /// # use rs_ali_oss::types::request::PutBucketEncryptionRequestBuilder;
    /// # use rs_ali_oss::types::ServerSideEncryption;
    /// # async fn example(client: OssClient) -> Result<()> {
    /// let request = PutBucketEncryptionRequestBuilder::new()
    ///     .bucket(BucketName::new("my-bucket")?)
    ///     .encryption(ServerSideEncryption::AES256)
    ///     .build()?;
    /// client.put_bucket_encryption(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn put_bucket_encryption(
        &self,
        request: PutBucketEncryptionRequest,
    ) -> Result<PutBucketEncryptionResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[("encryption", "")])?;
        let resource_path = format!("/{}/", request.bucket);

        let rule = EncryptionRuleXml {
            apply_server_side_encryption_by_default: request.encryption,
        };

        let config = EncryptionConfigurationXml { rule };
        let xml_body = serialize_xml(&config)?;
        let mut http_req = self
            .http_client()
            .request(Method::PUT, url)
            .header("content-type", "application/xml");
        http_req = http_req.body(xml_body);
        let http_req = http_req.build()?;

        let response = self.execute(http_req, &resource_path).await?;
        let request_id = header_opt(&response, "x-oss-request-id");
        Ok(PutBucketEncryptionResponse { request_id })
    }

    /// Get the encryption configuration of a bucket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rs_ali_oss::*;
    /// # use rs_ali_oss::types::request::GetBucketEncryptionRequestBuilder;
    /// # async fn example(client: OssClient) -> Result<()> {
    /// let request = GetBucketEncryptionRequestBuilder::new()
    ///     .bucket(BucketName::new("my-bucket")?)
    ///     .build()?;
    /// let response = client.get_bucket_encryption(request).await?;
    /// println!("Encryption: {}", response.rule.apply_server_side_encryption_by_default);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_bucket_encryption(
        &self,
        request: GetBucketEncryptionRequest,
    ) -> Result<GetBucketEncryptionResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[("encryption", "")])?;
        let resource_path = format!("/{}/", request.bucket);
        let http_req = self.http_client().request(Method::GET, url).build()?;
        let response = self.execute(http_req, &resource_path).await?;
        let body = response.text().await?;
        let resp: GetBucketEncryptionResponse = parse_xml(&body)?;
        Ok(resp)
    }

    /// Delete the encryption configuration of a bucket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rs_ali_oss::*;
    /// # use rs_ali_oss::types::request::DeleteBucketEncryptionRequestBuilder;
    /// # async fn example(client: OssClient) -> Result<()> {
    /// let request = DeleteBucketEncryptionRequestBuilder::new()
    ///     .bucket(BucketName::new("my-bucket")?)
    ///     .build()?;
    /// client.delete_bucket_encryption(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_bucket_encryption(
        &self,
        request: DeleteBucketEncryptionRequest,
    ) -> Result<DeleteBucketEncryptionResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[("encryption", "")])?;
        let resource_path = format!("/{}/", request.bucket);
        let http_req = self.http_client().request(Method::DELETE, url).build()?;
        let response = self.execute(http_req, &resource_path).await?;
        let request_id = header_opt(&response, "x-oss-request-id");
        Ok(DeleteBucketEncryptionResponse { request_id })
    }

    /// Set the logging configuration of a bucket.
    ///
    /// Logging enables you to store access logs to a target bucket with a specified prefix.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rs_ali_oss::*;
    /// # use rs_ali_oss::types::request::PutBucketLoggingRequestBuilder;
    /// # async fn example(client: OssClient) -> Result<()> {
    /// let request = PutBucketLoggingRequestBuilder::new()
    ///     .bucket(BucketName::new("my-bucket")?)
    ///     .target_bucket(BucketName::new("log-bucket")?)
    ///     .target_prefix("logs/")
    ///     .build()?;
    /// client.put_bucket_logging(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn put_bucket_logging(
        &self,
        request: PutBucketLoggingRequest,
    ) -> Result<PutBucketLoggingResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[("logging", "")])?;
        let resource_path = format!("/{}/", request.bucket);

        let logging_enabled = LoggingEnabledXml {
            target_bucket: request.target_bucket.to_string(),
            target_prefix: request.target_prefix.unwrap_or_default(),
        };

        let config = LoggingConfigurationXml { logging_enabled };
        let xml_body = serialize_xml(&config)?;
        let mut http_req = self
            .http_client()
            .request(Method::PUT, url)
            .header("content-type", "application/xml");
        http_req = http_req.body(xml_body);
        let http_req = http_req.build()?;

        let response = self.execute(http_req, &resource_path).await?;
        let request_id = header_opt(&response, "x-oss-request-id");
        Ok(PutBucketLoggingResponse { request_id })
    }

    /// Get the logging configuration of a bucket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rs_ali_oss::*;
    /// # use rs_ali_oss::types::request::GetBucketLoggingRequestBuilder;
    /// # async fn example(client: OssClient) -> Result<()> {
    /// let request = GetBucketLoggingRequestBuilder::new()
    ///     .bucket(BucketName::new("my-bucket")?)
    ///     .build()?;
    /// let response = client.get_bucket_logging(request).await?;
    /// if let Some(logging) = &response.logging_enabled {
    ///     println!("Target bucket: {}", logging.target_bucket);
    ///     println!("Target prefix: {}", logging.target_prefix);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_bucket_logging(
        &self,
        request: GetBucketLoggingRequest,
    ) -> Result<GetBucketLoggingResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[("logging", "")])?;
        let resource_path = format!("/{}/", request.bucket);
        let http_req = self.http_client().request(Method::GET, url).build()?;
        let response = self.execute(http_req, &resource_path).await?;
        let body = response.text().await?;
        let resp: GetBucketLoggingResponse = parse_xml(&body)?;
        Ok(resp)
    }

    /// Delete the logging configuration of a bucket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rs_ali_oss::*;
    /// # use rs_ali_oss::types::request::DeleteBucketLoggingRequestBuilder;
    /// # async fn example(client: OssClient) -> Result<()> {
    /// let request = DeleteBucketLoggingRequestBuilder::new()
    ///     .bucket(BucketName::new("my-bucket")?)
    ///     .build()?;
    /// client.delete_bucket_logging(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_bucket_logging(
        &self,
        request: DeleteBucketLoggingRequest,
    ) -> Result<DeleteBucketLoggingResponse> {
        let url = self.build_url(Some(&request.bucket), None, &[("logging", "")])?;
        let resource_path = format!("/{}/", request.bucket);
        let http_req = self.http_client().request(Method::DELETE, url).build()?;
        let response = self.execute(http_req, &resource_path).await?;
        let request_id = header_opt(&response, "x-oss-request-id");
        Ok(DeleteBucketLoggingResponse { request_id })
    }
}
