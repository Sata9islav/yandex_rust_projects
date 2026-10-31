use std::sync::Arc;

use actix_web::{HttpMessage, HttpRequest, HttpResponse, web};
use serde::Deserialize;

use crate::{
    application::{auth_service::AuthService, blog_service::BlogService},
    domain::post::{CreatePostRequest, UpdatePostRequest},
    domain::user::{LoginRequest, RegisterRequest},
    presentation::error::ApiError,
    presentation::middleware::AuthenticatedUser,
};

pub async fn register(
    service: web::Data<Arc<AuthService>>,
    request: web::Json<RegisterRequest>,
) -> Result<HttpResponse, ApiError> {
    let result = service.register(request.into_inner()).await?;
    Ok(HttpResponse::Created().json(result))
}

pub async fn login(
    service: web::Data<Arc<AuthService>>,
    request: web::Json<LoginRequest>,
) -> Result<HttpResponse, ApiError> {
    let result = service.login(request.into_inner()).await?;
    Ok(HttpResponse::Ok().json(result))
}

pub async fn get_post(
    service: web::Data<Arc<BlogService>>,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let post = service.get_post(id.into_inner()).await?;

    Ok(HttpResponse::Ok().json(post))
}

#[derive(Debug, Deserialize)]
pub struct ListPostQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_posts(
    service: web::Data<Arc<BlogService>>,
    query: web::Query<ListPostQuery>,
) -> Result<HttpResponse, ApiError> {
    let limit = query.limit.unwrap_or(10);
    let offset = query.offset.unwrap_or(0);

    let (posts, total) = service.list_posts(limit, offset).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "posts": posts,
        "total": total,
        "limit": limit,
        "offset": offset,
    })))
}

pub async fn create_post(
    service: web::Data<Arc<BlogService>>,
    http_request: HttpRequest,
    request: web::Json<CreatePostRequest>,
) -> Result<HttpResponse, ApiError> {
    let user_id = authenticated_user_id(&http_request)?;

    let post = service.create_post(user_id, request.into_inner()).await?;

    Ok(HttpResponse::Created().json(post))
}

pub async fn update_post(
    service: web::Data<Arc<BlogService>>,
    http_request: HttpRequest,
    id: web::Path<i64>,
    request: web::Json<UpdatePostRequest>,
) -> Result<HttpResponse, ApiError> {
    let user_id = authenticated_user_id(&http_request)?;

    let post = service
        .update_post(id.into_inner(), user_id, request.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(post))
}

pub async fn delete_post(
    service: web::Data<Arc<BlogService>>,
    http_request: HttpRequest,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let user_id = authenticated_user_id(&http_request)?;

    service.delete_post(id.into_inner(), user_id).await?;

    Ok(HttpResponse::NoContent().finish())
}

fn authenticated_user_id(request: &HttpRequest) -> Result<i64, ApiError> {
    let extensions = request.extensions();

    extensions
        .get::<AuthenticatedUser>()
        .map(|user| user.user_id)
        .ok_or(ApiError(
            crate::domain::error::DomainError::InvalidCredentials,
        ))
}
