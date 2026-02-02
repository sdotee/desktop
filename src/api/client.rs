use crate::config::Config;
use crate::error::{AppError, Result};
use reqwest::blocking::Client as HttpClient;
use see_sdk::{
    client::Client,
    config::Config as SdkConfig,
    domain::DomainService,
    file::{models::FileUploadResponse, FileService},
    text::models::{CreateTextResponse, DeleteTextRequest},
    text::TextService,
    url::{
        builder::UrlShortenerRequestBuilder,
        models::{DeleteRequest as DeleteUrlRequest, ShortenResponse},
        ShortenService,
    },
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

/// Text type for S.EE API
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TextType {
    #[default]
    PlainText,
    SourceCode,
    Markdown,
}

impl TextType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TextType::PlainText => "plain_text",
            TextType::SourceCode => "source_code",
            TextType::Markdown => "markdown",
        }
    }
}

/// Extended text request with domain and type support
#[derive(Debug, Serialize)]
struct ExtendedCreateTextRequest {
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_slug: Option<String>,
}

pub struct ApiClient {
    client: Client,
    http_client: HttpClient,
    base_url: String,
    api_key: String,
}

impl ApiClient {
    pub fn new(config: &Config) -> Result<Self> {
        let api_key = config.api_key().ok_or(AppError::NoApiKey)?.to_string();
        let base_url = config.base_url().to_string();

        let sdk_config = SdkConfig::new(&base_url)
            .with_api_key(&api_key)
            .with_timeout(Duration::from_secs(config.timeout()));

        let client = Client::new(sdk_config).map_err(|e| AppError::Api(e.to_string()))?;

        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(config.timeout()))
            .build()
            .map_err(|e: reqwest::Error| AppError::Api(e.to_string()))?;

        Ok(Self {
            client,
            http_client,
            base_url,
            api_key,
        })
    }

    // Domain listing
    pub fn get_url_domains(&self) -> Result<Vec<String>> {
        let response = self.client.list().map_err(AppError::from)?;
        Ok(response.data.domains)
    }

    pub fn get_text_domains(&self) -> Result<Vec<String>> {
        let response = self.client.get_text_domains().map_err(AppError::from)?;
        Ok(response.data.domains)
    }

    pub fn get_file_domains(&self) -> Result<Vec<String>> {
        let response = self.client.get_file_domains().map_err(AppError::from)?;
        Ok(response.data.domains)
    }

    // URL operations
    pub fn shorten_url(
        &self,
        url: &str,
        domain: Option<&str>,
        slug: Option<&str>,
    ) -> Result<ShortenResponse> {
        let mut builder =
            UrlShortenerRequestBuilder::new(url).map_err(|e| AppError::Api(e.to_string()))?;

        if let Some(d) = domain {
            builder = builder.with_domain(d);
        }
        if let Some(s) = slug {
            builder = builder.with_custom_alias(s);
        }

        let request = builder.build();
        self.client.shorten(request).map_err(AppError::from)
    }

    pub fn delete_url(&self, domain: &str, slug: &str) -> Result<()> {
        let request = DeleteUrlRequest {
            domain: domain.to_string(),
            slug: slug.to_string(),
        };
        self.client.delete(request).map_err(AppError::from)?;
        Ok(())
    }

    // Text operations
    pub fn create_text(
        &self,
        content: &str,
        title: &str,
        domain: Option<&str>,
        text_type: Option<TextType>,
    ) -> Result<CreateTextResponse> {
        // Use direct API call to support domain and type
        let request = ExtendedCreateTextRequest {
            content: content.to_string(),
            title: Some(title.to_string()), // API requires title
            domain: domain.map(String::from),
            text_type: text_type.map(|t| t.as_str().to_string()),
            custom_slug: None,
        };

        let response = self
            .http_client
            .post(format!("{}/text", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .map_err(|e: reqwest::Error| AppError::Api(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().unwrap_or_default();
            return Err(AppError::Api(format!("API error {}: {}", status, text)));
        }

        response
            .json::<CreateTextResponse>()
            .map_err(|e: reqwest::Error| AppError::Api(e.to_string()))
    }

    pub fn delete_text(&self, domain: &str, slug: &str) -> Result<()> {
        let request = DeleteTextRequest {
            domain: domain.to_string(),
            slug: slug.to_string(),
        };
        self.client.delete_text(request).map_err(AppError::from)?;
        Ok(())
    }

    // File operations
    pub fn upload_file(&self, path: &Path) -> Result<FileUploadResponse> {
        self.client.upload_file(path).map_err(AppError::from)
    }

    pub fn delete_file(&self, key: &str) -> Result<()> {
        self.client.delete_file(key).map_err(AppError::from)?;
        Ok(())
    }
}

/// Async bridge for calling blocking SDK methods from GTK main loop
pub mod async_bridge {
    use super::*;
    use async_channel::{bounded, Receiver, Sender};
    use gtk::gio;
    use std::path::PathBuf;

    pub enum ApiRequest {
        // Domain listing
        GetUrlDomains,
        GetTextDomains,
        GetFileDomains,
        // URL operations
        ShortenUrl {
            url: String,
            domain: Option<String>,
            slug: Option<String>,
        },
        DeleteUrl {
            domain: String,
            slug: String,
        },
        // Text operations
        CreateText {
            content: String,
            title: String,
            domain: Option<String>,
            text_type: Option<TextType>,
        },
        DeleteText {
            domain: String,
            slug: String,
        },
        // File operations
        UploadFile {
            path: PathBuf,
        },
        DeleteFile {
            key: String,
        },
    }

    pub enum ApiResponse {
        // Domain listing
        GetUrlDomains(Result<Vec<String>>),
        GetTextDomains(Result<Vec<String>>),
        GetFileDomains(Result<Vec<String>>),
        // URL operations
        ShortenUrl(Result<ShortenResponse>),
        DeleteUrl(Result<()>),
        // Text operations
        CreateText(Result<CreateTextResponse>),
        DeleteText(Result<()>),
        // File operations
        UploadFile(Result<FileUploadResponse>),
        DeleteFile(Result<()>),
    }

    pub fn spawn_api_call(config: Config, request: ApiRequest) -> Receiver<ApiResponse> {
        let (sender, receiver): (Sender<ApiResponse>, Receiver<ApiResponse>) = bounded(1);

        gio::spawn_blocking(move || {
            let response = match ApiClient::new(&config) {
                Ok(client) => match request {
                    // Domain listing
                    ApiRequest::GetUrlDomains => {
                        ApiResponse::GetUrlDomains(client.get_url_domains())
                    }
                    ApiRequest::GetTextDomains => {
                        ApiResponse::GetTextDomains(client.get_text_domains())
                    }
                    ApiRequest::GetFileDomains => {
                        ApiResponse::GetFileDomains(client.get_file_domains())
                    }
                    // URL operations
                    ApiRequest::ShortenUrl { url, domain, slug } => {
                        ApiResponse::ShortenUrl(client.shorten_url(
                            &url,
                            domain.as_deref(),
                            slug.as_deref(),
                        ))
                    }
                    ApiRequest::DeleteUrl { domain, slug } => {
                        ApiResponse::DeleteUrl(client.delete_url(&domain, &slug))
                    }
                    // Text operations
                    ApiRequest::CreateText {
                        content,
                        title,
                        domain,
                        text_type,
                    } => ApiResponse::CreateText(client.create_text(
                        &content,
                        &title,
                        domain.as_deref(),
                        text_type,
                    )),
                    ApiRequest::DeleteText { domain, slug } => {
                        ApiResponse::DeleteText(client.delete_text(&domain, &slug))
                    }
                    // File operations
                    ApiRequest::UploadFile { path } => {
                        ApiResponse::UploadFile(client.upload_file(&path))
                    }
                    ApiRequest::DeleteFile { key } => {
                        ApiResponse::DeleteFile(client.delete_file(&key))
                    }
                },
                Err(e) => match request {
                    ApiRequest::GetUrlDomains => ApiResponse::GetUrlDomains(Err(e)),
                    ApiRequest::GetTextDomains => ApiResponse::GetTextDomains(Err(e)),
                    ApiRequest::GetFileDomains => ApiResponse::GetFileDomains(Err(e)),
                    ApiRequest::ShortenUrl { .. } => ApiResponse::ShortenUrl(Err(e)),
                    ApiRequest::DeleteUrl { .. } => ApiResponse::DeleteUrl(Err(e)),
                    ApiRequest::CreateText { .. } => ApiResponse::CreateText(Err(e)),
                    ApiRequest::DeleteText { .. } => ApiResponse::DeleteText(Err(e)),
                    ApiRequest::UploadFile { .. } => ApiResponse::UploadFile(Err(e)),
                    ApiRequest::DeleteFile { .. } => ApiResponse::DeleteFile(Err(e)),
                },
            };

            let _ = sender.send_blocking(response);
        });

        receiver
    }
}
