use gloo_net::http::{Request, Response};
use serde::{Serialize, de::DeserializeOwned};
use wasm_bindgen::JsValue;

use crate::models::{AuthResponse, Post, PostList};

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

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}

async fn parse_response<T: DeserializeOwned>(response: Response) -> Result<T, JsValue> {
    let status = response.status();

    if !(200..300).contains(&status) {
        let message = response.text().await.map_err(js_error)?;

        return Err(JsValue::from_str(&format!("HTTP {status}: {message}")));
    }

    response.json::<T>().await.map_err(js_error)
}

pub async fn register(
    server_url: &str,
    username: &str,
    email: &str,
    password: &str,
) -> Result<AuthResponse, JsValue> {
    let url = format!("{}/api/auth/register", server_url.trim_end_matches('/'));

    let request = Request::post(&url)
        .json(&RegisterRequest {
            username,
            email,
            password,
        })
        .map_err(js_error)?;

    let response = request.send().await.map_err(js_error)?;

    parse_response(response).await
}

pub async fn login(
    server_url: &str,
    username: &str,
    password: &str,
) -> Result<AuthResponse, JsValue> {
    let url = format!("{}/api/auth/login", server_url.trim_end_matches('/'));

    let request = Request::post(&url)
        .json(&LoginRequest { username, password })
        .map_err(js_error)?;

    let response = request.send().await.map_err(js_error)?;

    parse_response(response).await
}

pub async fn load_posts(server_url: &str, limit: i64, offset: i64) -> Result<PostList, JsValue> {
    let url = format!(
        "{}/api/posts?limit={limit}&offset={offset}",
        server_url.trim_end_matches('/')
    );

    let response = Request::get(&url).send().await.map_err(js_error)?;

    parse_response(response).await
}

pub async fn create_post(
    server_url: &str,
    token: &str,
    title: &str,
    content: &str,
) -> Result<Post, JsValue> {
    let url = format!("{}/api/posts", server_url.trim_end_matches('/'));

    let authorization = format!("Bearer {token}");

    let request = Request::post(&url)
        .header("Authorization", &authorization)
        .json(&PostRequest { title, content })
        .map_err(js_error)?;

    let response = request.send().await.map_err(js_error)?;

    parse_response(response).await
}

pub async fn update_post(
    server_url: &str,
    token: &str,
    id: i64,
    title: &str,
    content: &str,
) -> Result<Post, JsValue> {
    let url = format!("{}/api/posts/{id}", server_url.trim_end_matches('/'));

    let authorization = format!("Bearer {token}");

    let request = Request::put(&url)
        .header("Authorization", &authorization)
        .json(&PostRequest { title, content })
        .map_err(js_error)?;

    let response = request.send().await.map_err(js_error)?;

    parse_response(response).await
}

pub async fn delete_post(server_url: &str, token: &str, id: i64) -> Result<(), JsValue> {
    let url = format!("{}/api/posts/{id}", server_url.trim_end_matches('/'));

    let authorization = format!("Bearer {token}");

    let response = Request::delete(&url)
        .header("Authorization", &authorization)
        .send()
        .await
        .map_err(js_error)?;

    let status = response.status();

    if !(200..300).contains(&status) {
        let message = response.text().await.map_err(js_error)?;

        return Err(JsValue::from_str(&format!("HTTP {status}: {message}")));
    }

    Ok(())
}
