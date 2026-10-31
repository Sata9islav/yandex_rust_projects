use super::*;

const TEST_SECRET: &str = "test_secret_key_with_at_least_32_byttes";

#[test]
fn test_generate_and_verify_token() {
    let jwt = JwtService::new(TEST_SECRET).unwrap();

    let token = jwt.generate_token(42, "alice").unwrap();
    let claims = jwt.verify_token(&token).unwrap();

    assert_eq!(claims.user_id, 42);
    assert_eq!(claims.username, "alice");
}

#[test]
fn test_invalid_token() {
    let jwt = JwtService::new(TEST_SECRET).unwrap();
    assert!(jwt.verify_token("invalid.token.value").is_err());
}

#[test]
fn wrong_secret() {
    let jwt1 = JwtService::new(TEST_SECRET).unwrap();
    let jwt2 = JwtService::new("another_secret_at_least_32_bytes").unwrap();

    let token = jwt1.generate_token(42, "alice").unwrap();
    assert!(jwt2.verify_token(&token).is_err());
}

#[test]
fn test_short_secret() {
    assert!(JwtService::new("short").is_err());
}
