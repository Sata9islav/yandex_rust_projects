use std::sync::Arc;

use actix_web::{Error, HttpMessage, dev::ServiceRequest, error::ErrorUnauthorized, web};

use actix_web_httpauth::extractors::bearer::BearerAuth;

use crate::infrastructure::jwt::JwtService;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: i64,
}

pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let jwt_service = match req.app_data::<web::Data<Arc<JwtService>>>() {
        Some(service) => service,
        None => {
            return Err((ErrorUnauthorized("Authentication service unavailable"), req));
        }
    };

    let claims = match jwt_service.verify_token(credentials.token()) {
        Ok(claims) => claims,
        Err(_) => {
            return Err((ErrorUnauthorized("Invalid or Expired token"), req));
        }
    };

    req.extensions_mut().insert(AuthenticatedUser {
        user_id: claims.user_id,
    });

    Ok(req)
}
