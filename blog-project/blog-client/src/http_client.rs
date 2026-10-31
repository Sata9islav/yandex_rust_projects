use reqwest::StatusCode;
use serde::Serialize;

use crate::models::{Post, PostList};
use crate::{error::BlogClientError, models::AuthResponse};

#[derive(Serialize)]
struct RegisterRequest<'a> {
    username: &'a str,
    email: &'a str,
    password: &'a str,
}

#[derive(Serialize)]
struct LoginRequest<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Serialize)]
struct PostRequest<'a> {
    title: &'a str,
    content: &'a str,
}

pub async fn create_post(
    client: &reqwest::Client,
    base_url: &str,
    token: &str,
    title: &str,
    content: &str,
) -> Result<Post, BlogClientError> {
    let url = format!("{}/api/posts", base_url.trim_end_matches('/'));

    let response = client
        .post(url)
        .bearer_auth(token)
        .json(&PostRequest { title, content })
        .send()
        .await?;

    parse_response(response).await
}

pub async fn get_post(
    client: &reqwest::Client,
    base_url: &str,
    id: i64,
) -> Result<Post, BlogClientError> {
    let url = format!("{}/api/posts/{id}", base_url.trim_end_matches('/'));

    let response = client.get(url).send().await?;

    parse_response(response).await
}

pub async fn update_post(
    client: &reqwest::Client,
    base_url: &str,
    token: &str,
    id: i64,
    title: &str,
    content: &str,
) -> Result<Post, BlogClientError> {
    let url = format!("{}/api/posts/{id}", base_url.trim_end_matches('/'));

    let response = client
        .put(url)
        .bearer_auth(token)
        .json(&PostRequest { title, content })
        .send()
        .await?;

    parse_response(response).await
}

pub async fn delete_post(
    client: &reqwest::Client,
    base_url: &str,
    token: &str,
    id: i64,
) -> Result<(), BlogClientError> {
    let url = format!("{}/api/posts/{id}", base_url.trim_end_matches('/'));

    let response = client.delete(url).bearer_auth(token).send().await?;

    check_response(response).await
}

pub async fn list_posts(
    client: &reqwest::Client,
    base_url: &str,
    limit: i64,
    offset: i64,
) -> Result<PostList, BlogClientError> {
    let url = format!("{}/api/posts", base_url.trim_end_matches('/'));

    let response = client
        .get(url)
        .query(&[("limit", limit), ("offset", offset)])
        .send()
        .await?;

    parse_response(response).await
}

async fn check_response(response: reqwest::Response) -> Result<(), BlogClientError> {
    let status = response.status();

    match status {
        StatusCode::NOT_FOUND => Err(BlogClientError::NotFound),
        StatusCode::UNAUTHORIZED => Err(BlogClientError::Unathorized),
        StatusCode::FORBIDDEN => Err(BlogClientError::Forbidden),
        StatusCode::BAD_REQUEST | StatusCode::CONFLICT => {
            Err(BlogClientError::InvalidRequest(response.text().await?))
        }

        _ if status.is_success() => Ok(()),
        _ => Err(BlogClientError::UnexpectedResponse(format!(
            "HTTP {status}: {}",
            response.text().await?
        ))),
    }
}

async fn parse_response<T: serde::de::DeserializeOwned>(
    response: reqwest::Response,
) -> Result<T, BlogClientError> {
    check_status_before_parse(response).await
}

async fn check_status_before_parse<T: serde::de::DeserializeOwned>(
    response: reqwest::Response,
) -> Result<T, BlogClientError> {
    let status = response.status();
    if status.is_success() {
        return Ok(response.json::<T>().await?);
    }

    check_response(response).await?;

    Err(BlogClientError::UnexpectedResponse(
        "Expected JSON response".into(),
    ))
}

pub async fn register(
    client: &reqwest::Client,
    base_url: &str,
    username: &str,
    email: &str,
    password: &str,
) -> Result<AuthResponse, BlogClientError> {
    let url = format!("{}/api/auth/register", base_url.trim_end_matches('/'));

    let response = client
        .post(url)
        .json(&RegisterRequest {
            username,
            email,
            password,
        })
        .send()
        .await?;
    parse_auth_response(response).await
}

pub async fn login(
    client: &reqwest::Client,
    base_url: &str,
    username: &str,
    password: &str,
) -> Result<AuthResponse, BlogClientError> {
    let url = format!("{}/api/auth/login", base_url.trim_end_matches('/'));

    let response = client
        .post(url)
        .json(&LoginRequest { username, password })
        .send()
        .await?;

    parse_auth_response(response).await
}

async fn parse_auth_response(response: reqwest::Response) -> Result<AuthResponse, BlogClientError> {
    let status = response.status();

    match status {
        StatusCode::UNAUTHORIZED => Err(BlogClientError::Unathorized),
        StatusCode::FORBIDDEN => Err(BlogClientError::Forbidden),
        StatusCode::BAD_REQUEST | StatusCode::CONFLICT => {
            let message = response.text().await?;
            Err(BlogClientError::InvalidRequest(message))
        }

        _ if status.is_success() => Ok(response.json::<AuthResponse>().await?),

        _ => {
            let message = response.text().await?;
            Err(BlogClientError::UnexpectedResponse(format!(
                "HTTP {status}: {message}"
            )))
        }
    }
}
