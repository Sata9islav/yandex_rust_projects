use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    application::auth_service::AuthService,
    data::user_repository::UserRepository,
    domain::{
        error::DomainError,
        user::{LoginRequest, RegisterRequest},
    },
    infrastructure::jwt::JwtService,
};

#[tokio::test]
async fn test_auth_service() -> anyhow::Result<()> {
    let env_path = concat!(env!("CARGO_MANIFEST_DIR"), "/.env");
    dotenvy::from_path(env_path)?;

    let database_url = std::env::var("DATABASE_URL")?;
    let pool = sqlx::PgPool::connect(&database_url).await?;

    let user_repository = UserRepository::new(pool.clone());

    let jwt_service = Arc::new(JwtService::new("test_secret_key_with_at_least_32_bytes")?);

    let auth_service = AuthService::new(user_repository, jwt_service.clone());

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();

    let username = format!("auth_test_{timestamp}");
    let email = format!("{username}@example.com");
    let password = "TestPassword123!";

    let registered = auth_service
        .register(RegisterRequest {
            username: username.clone(),
            email: email.clone(),
            password: password.to_string(),
        })
        .await?;
    assert_eq!(registered.user.username, username);
    assert_eq!(registered.user.email, email);
    assert!(!registered.token.is_empty());

    let claims = jwt_service.verify_token(&registered.token)?;
    assert_eq!(claims.user_id, registered.user.id);
    assert_eq!(claims.username, username);

    let (stored_hash,): (String,) = sqlx::query_as("SELECT password_hash FROM users WHERE id = $1")
        .bind(registered.user.id)
        .fetch_one(&pool)
        .await?;
    assert_ne!(stored_hash, password);
    assert!(stored_hash.starts_with("$argon2"));

    let logged_in = auth_service
        .login(LoginRequest {
            username: username.clone(),
            password: password.to_string(),
        })
        .await?;
    assert_eq!(logged_in.user.id, registered.user.id);
    assert!(!logged_in.token.is_empty());

    let invalid_login = auth_service
        .login(LoginRequest {
            username: username.clone(),
            password: "wrong_password".to_string(),
        })
        .await;
    assert!(matches!(
        invalid_login,
        Err(DomainError::InvalidCredentials)
    ));

    let duplicate = auth_service
        .register(RegisterRequest {
            username: username.clone(),
            email: email.clone(),
            password: password.to_string(),
        })
        .await;
    assert!(matches!(duplicate, Err(DomainError::UserAlreadyExists)));

    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(registered.user.id)
        .execute(&pool)
        .await?;

    pool.close().await;

    Ok(())
}
