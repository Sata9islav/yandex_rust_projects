mod api;
mod models;
mod storage;

use wasm_bindgen::prelude::*;

use crate::storage::{get_token_from_storage, remove_token_from_storage, save_token_to_storage};

#[wasm_bindgen]
pub struct BlogApp {
    server_url: String,
    token: Option<String>,
    user_id: Option<i64>,
}

#[wasm_bindgen]
impl BlogApp {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<BlogApp, JsValue> {
        let token = get_token_from_storage()?;
        let user_id = if token.is_some() {
            storage::get_user_id()?
        } else {
            None
        };

        Ok(Self {
            server_url: "http://127.0.0.1:8080".to_string(),
            token,
            user_id,
        })
    }

    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    pub fn server_url(&self) -> String {
        self.server_url.clone()
    }

    pub fn current_user_id(&self) -> Option<i64> {
        self.user_id
    }

    pub fn logout(&mut self) -> Result<(), JsValue> {
        remove_token_from_storage()?;
        storage::remove_user_id()?;

        self.token = None;
        self.user_id = None;

        Ok(())
    }

    pub async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<JsValue, JsValue> {
        if username.trim().is_empty() || email.trim().is_empty() || password.is_empty() {
            return Err(JsValue::from_str(
                "Username, email and password are required",
            ));
        }

        let auth = api::register(&self.server_url, username, email, password).await?;
        save_token_to_storage(&auth.token)?;
        storage::save_user_id(auth.user.id)?;

        self.token = Some(auth.token.clone());
        self.user_id = Some(auth.user.id);

        serde_wasm_bindgen::to_value(&auth).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub async fn login(&mut self, username: &str, password: &str) -> Result<JsValue, JsValue> {
        if username.trim().is_empty() || password.is_empty() {
            return Err(JsValue::from_str("Username and password are required"));
        }

        let auth = api::login(&self.server_url, username, password).await?;

        save_token_to_storage(&auth.token)?;
        storage::save_user_id(auth.user.id)?;

        self.token = Some(auth.token.clone());
        self.user_id = Some(auth.user.id);

        serde_wasm_bindgen::to_value(&auth).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub async fn load_posts(&self) -> Result<JsValue, JsValue> {
        let posts = api::load_posts(&self.server_url, 20, 0).await?;

        serde_wasm_bindgen::to_value(&posts).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub async fn create_post(&self, title: &str, content: &str) -> Result<JsValue, JsValue> {
        if title.trim().is_empty() || content.trim().is_empty() {
            return Err(JsValue::from_str("Title and content are required"));
        }

        let token = self
            .token
            .as_deref()
            .ok_or_else(|| JsValue::from_str("Authentication required"))?;

        let post = api::create_post(&self.server_url, token, title, content).await?;

        serde_wasm_bindgen::to_value(&post).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub async fn update_post(
        &self,
        id: i64,
        title: &str,
        content: &str,
    ) -> Result<JsValue, JsValue> {
        if id <= 0 {
            return Err(JsValue::from_str("Invalid post ID"));
        }

        if title.trim().is_empty() || content.trim().is_empty() {
            return Err(JsValue::from_str("Title and content are required"));
        }

        let token = self
            .token
            .as_deref()
            .ok_or_else(|| JsValue::from_str("Authentication required"))?;

        let post = api::update_post(&self.server_url, token, id, title, content).await?;

        serde_wasm_bindgen::to_value(&post).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub async fn delete_post(&self, id: i64) -> Result<JsValue, JsValue> {
        if id <= 0 {
            return Err(JsValue::from_str("Invalid post ID"));
        }

        let token = self
            .token
            .as_deref()
            .ok_or_else(|| JsValue::from_str("Authentication required"))?;

        api::delete_post(&self.server_url, token, id).await?;

        Ok(JsValue::NULL)
    }
}
