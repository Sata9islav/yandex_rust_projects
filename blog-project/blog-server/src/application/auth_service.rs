use std::sync::Arc;

use argon2::{Argon2, PasswordHasher, PasswordVerifier, password_hash::phc::PasswordHash};

use serde::Serialize;

use crate::{
    data::user_repository::UserRepository,
    domain::{
        error::DomainError,
        user::{LoginRequest, RegisterRequest, User},
    },
    infrastructure::jwt::JwtService,
};

#[derive(Debug, Serialize)]
pub struct AuthUser {
    pub id: i64,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResult {
    pub token: String,
    pub user: AuthUser,
}

pub struct AuthService {
    user_repository: UserRepository,
    jwt_service: Arc<JwtService>,
}

impl AuthService {
    pub fn new(user_repository: UserRepository, jwt_service: Arc<JwtService>) -> Self {
        Self {
            user_repository,
            jwt_service,
        }
    }

    pub async fn register(&self, request: RegisterRequest) -> Result<AuthResult, DomainError> {
        if request.username.trim().is_empty()
            || request.email.trim().is_empty()
            || request.password.is_empty()
        {
            return Err(DomainError::InvalidInput(
                "Required fields cannot be empty".into(),
            ));
        }

        let password = request.password;
        let password_hash = tokio::task::spawn_blocking(move || {
            Argon2::default()
                .hash_password(password.as_bytes())
                .map(|hash| hash.to_string())
        })
        .await
        .map_err(|_| DomainError::Internal)?
        .map_err(|_| DomainError::Internal)?;

        let user = self
            .user_repository
            .create(&request.username, &request.email, &password_hash)
            .await
            .map_err(|error| match &error {
                sqlx::Error::Database(db_error) if db_error.code().as_deref() == Some("23505") => {
                    DomainError::UserAlreadyExists
                }
                _ => DomainError::Internal,
            })?;
        self.create_auth_result(user)
    }

    pub async fn login(&self, request: LoginRequest) -> Result<AuthResult, DomainError> {
        let user = self
            .user_repository
            .find_by_username(&request.username)
            .await
            .map_err(|_| DomainError::Internal)?
            .ok_or(DomainError::InvalidCredentials)?;

        let password_hash = user.password_hash.clone();
        let password = request.password;

        let valid = tokio::task::spawn_blocking(move || {
            let parsed_hash = match PasswordHash::new(&password_hash) {
                Ok(hash) => hash,
                Err(_) => return false,
            };

            Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok()
        })
        .await
        .map_err(|_| DomainError::Internal)?;

        if !valid {
            return Err(DomainError::InvalidCredentials);
        }

        self.create_auth_result(user)
    }

    fn create_auth_result(&self, user: User) -> Result<AuthResult, DomainError> {
        let token = self
            .jwt_service
            .generate_token(user.id, &user.username)
            .map_err(|_| DomainError::Internal)?;

        Ok(AuthResult {
            token,
            user: AuthUser {
                id: user.id,
                username: user.username,
                email: user.email,
            },
        })
    }
}

#[cfg(test)]
#[path = "tests_auth_service.rs"]
mod tests;
