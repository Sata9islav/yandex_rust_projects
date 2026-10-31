pub mod error;
pub mod grpc_client;
pub mod http_client;
pub mod models;

use error::BlogClientError;

#[derive(Debug, Clone)]
pub enum Transport {
    Http(String),
    Grpc(String),
}

pub struct BlogClient {
    transport: Transport,
    http_client: Option<reqwest::Client>,
    grpc_client:
        Option<blog_proto::blog::blog_service_client::BlogServiceClient<tonic::transport::Channel>>,
    token: Option<String>,
}

impl BlogClient {
    pub async fn new(transport: Transport) -> Result<Self, BlogClientError> {
        match &transport {
            Transport::Http(_) => Ok(Self {
                transport,
                http_client: Some(reqwest::Client::new()),
                grpc_client: None,
                token: None,
            }),

            Transport::Grpc(address) => {
                let client = blog_proto::blog::blog_service_client::BlogServiceClient::connect(
                    address.clone(),
                )
                .await?;

                Ok(Self {
                    transport: transport.clone(),
                    http_client: None,
                    grpc_client: Some(client),
                    token: None,
                })
            }
        }
    }
    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn get_token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<models::AuthResponse, BlogClientError> {
        let result = match &self.transport {
            Transport::Http(url) => {
                let client = self.http_client.as_ref().ok_or_else(|| {
                    BlogClientError::InvalidRequest("HTTP client not initialized".into())
                })?;

                http_client::register(client, url, username, email, password).await?
            }

            Transport::Grpc(_) => {
                let client = self.grpc_client.as_mut().ok_or_else(|| {
                    BlogClientError::InvalidRequest("gRPC client not initialized".into())
                })?;

                grpc_client::register(client, username, email, password).await?
            }
        };

        self.set_token(result.token.clone());

        Ok(result)
    }

    pub async fn login(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<models::AuthResponse, BlogClientError> {
        let result = match &self.transport {
            Transport::Http(url) => {
                let client = self.http_client.as_ref().ok_or_else(|| {
                    BlogClientError::InvalidRequest("HTTP client not initialized".into())
                })?;

                http_client::login(client, url, username, password).await?
            }

            Transport::Grpc(_) => {
                let client = self.grpc_client.as_mut().ok_or_else(|| {
                    BlogClientError::InvalidRequest("gRPC client not initialized".into())
                })?;

                grpc_client::login(client, username, password).await?
            }
        };

        self.set_token(result.token.clone());

        Ok(result)
    }

    fn required_token(&self) -> Result<&str, BlogClientError> {
        self.get_token().ok_or(BlogClientError::Unathorized)
    }

    pub async fn create_post(
        &mut self,
        title: &str,
        content: &str,
    ) -> Result<models::Post, BlogClientError> {
        let token = self.required_token()?.to_owned();

        match &self.transport {
            Transport::Http(url) => {
                let client = self.http_client.as_ref().ok_or_else(|| {
                    BlogClientError::InvalidRequest("HTTP client not initialized".into())
                })?;

                http_client::create_post(client, url, &token, title, content).await
            }
            Transport::Grpc(_) => {
                let client = self.grpc_client.as_mut().ok_or_else(|| {
                    BlogClientError::InvalidRequest("gRPC client not initialized".into())
                })?;

                grpc_client::create_post(client, &token, title, content).await
            }
        }
    }

    pub async fn get_post(&mut self, id: i64) -> Result<models::Post, BlogClientError> {
        match &self.transport {
            Transport::Http(url) => {
                let client = self.http_client.as_ref().ok_or_else(|| {
                    BlogClientError::InvalidRequest("HTTP client not initialized".into())
                })?;

                http_client::get_post(client, url, id).await
            }
            Transport::Grpc(_) => {
                let client = self.grpc_client.as_mut().ok_or_else(|| {
                    BlogClientError::InvalidRequest("gRPC client not initialized".into())
                })?;

                grpc_client::get_post(client, id).await
            }
        }
    }

    pub async fn update_post(
        &mut self,
        id: i64,
        title: &str,
        content: &str,
    ) -> Result<models::Post, BlogClientError> {
        let token = self.required_token()?.to_owned();

        match &self.transport {
            Transport::Http(url) => {
                let client = self.http_client.as_ref().ok_or_else(|| {
                    BlogClientError::InvalidRequest("HTTP client not initialized".into())
                })?;

                http_client::update_post(client, url, &token, id, title, content).await
            }
            Transport::Grpc(_) => {
                let client = self.grpc_client.as_mut().ok_or_else(|| {
                    BlogClientError::InvalidRequest("gRPC client not initialized".into())
                })?;

                grpc_client::update_post(client, &token, id, title, content).await
            }
        }
    }

    pub async fn delete_post(&mut self, id: i64) -> Result<(), BlogClientError> {
        let token = self.required_token()?.to_owned();

        match &self.transport {
            Transport::Http(url) => {
                let client = self.http_client.as_ref().ok_or_else(|| {
                    BlogClientError::InvalidRequest("HTTP client not initialized".into())
                })?;

                http_client::delete_post(client, url, &token, id).await
            }
            Transport::Grpc(_) => {
                let client = self.grpc_client.as_mut().ok_or_else(|| {
                    BlogClientError::InvalidRequest("gRPC client not initialized".into())
                })?;

                grpc_client::delete_post(client, &token, id).await
            }
        }
    }

    pub async fn list_posts(
        &mut self,
        limit: i64,
        offset: i64,
    ) -> Result<models::PostList, BlogClientError> {
        match &self.transport {
            Transport::Http(url) => {
                let client = self.http_client.as_ref().ok_or_else(|| {
                    BlogClientError::InvalidRequest("HTTP client not initialized".into())
                })?;

                http_client::list_posts(client, url, limit, offset).await
            }
            Transport::Grpc(_) => {
                let client = self.grpc_client.as_mut().ok_or_else(|| {
                    BlogClientError::InvalidRequest("gRPC client not initialized".into())
                })?;

                grpc_client::list_posts(client, limit, offset).await
            }
        }
    }
}
