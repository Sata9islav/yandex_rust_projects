use wasm_bindgen::JsValue;

const TOKEN_KEY: &str = "blog_token";
const USER_ID_KEY: &str = "blog_user_id";

fn storage() -> Result<web_sys::Storage, JsValue> {
    web_sys::window()
        .ok_or_else(|| JsValue::from_str("Window is not available"))?
        .local_storage()?
        .ok_or_else(|| JsValue::from_str("localStorage is not available"))
}

pub fn save_token_to_storage(token: &str) -> Result<(), JsValue> {
    storage()?.set_item(TOKEN_KEY, token)
}

pub fn get_token_from_storage() -> Result<Option<String>, JsValue> {
    storage()?.get_item(TOKEN_KEY)
}

pub fn remove_token_from_storage() -> Result<(), JsValue> {
    storage()?.remove_item(TOKEN_KEY)
}

pub fn save_user_id(id: i64) -> Result<(), JsValue> {
    storage()?.set_item(USER_ID_KEY, &id.to_string())
}

pub fn get_user_id() -> Result<Option<i64>, JsValue> {
    Ok(storage()?
        .get_item(USER_ID_KEY)?
        .and_then(|value| value.parse::<i64>().ok()))
}

pub fn remove_user_id() -> Result<(), JsValue> {
    storage()?.remove_item(USER_ID_KEY)
}
