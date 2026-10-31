use std::sync::Arc;

use tonic::{Request, Response, Status};

use blog_proto::blog::{
    self as proto, AuthResponse, ListPostsResponse,
    blog_service_server::BlogService as BlogServiceTrait,
};

use crate::{
    application::{auth_service::AuthService, blog_service::BlogService},
    domain::{
        error::DomainError,
        post::Post,
        user::{LoginRequest as DomainLoginRequest, RegisterRequest as DomainRegisterRequest},
    },
};

use crate::infrastructure::jwt::JwtService;

pub struct BlogGrpcService {
    auth_service: Arc<AuthService>,
    blog_service: Arc<BlogService>,
    jwt_service: Arc<JwtService>,
}

impl BlogGrpcService {
    pub fn new(
        auth_service: Arc<AuthService>,
        blog_service: Arc<BlogService>,
        jwt_service: Arc<JwtService>,
    ) -> Self {
        Self {
            auth_service,
            blog_service,
            jwt_service,
        }
    }

    pub fn authenticated_user<T>(&self, request: &Request<T>) -> Result<i64, Status> {
        let authorization = request
            .metadata()
            .get("authorization")
            .ok_or_else(|| Status::unauthenticated("Missing authorization header"))?;

        let authorization = authorization
            .to_str()
            .map_err(|_| Status::unauthenticated("Invalid authorization header"))?;

        let token = authorization
            .strip_prefix("Bearer ")
            .filter(|token| !token.is_empty())
            .ok_or_else(|| Status::unauthenticated("Expected Bearer token"))?;

        let claims = self
            .jwt_service
            .verify_token(token)
            .map_err(|_| Status::unauthenticated("Invalid or expired token"))?;

        Ok(claims.user_id)
    }
}

#[tonic::async_trait]
impl BlogServiceTrait for BlogGrpcService {
    async fn register(
        &self,
        request: Request<proto::RegisterRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let request = request.into_inner();

        let result = self
            .auth_service
            .register(DomainRegisterRequest {
                username: request.username,
                email: request.email,
                password: request.password,
            })
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(proto::AuthResponse {
            access_token: result.token,
            user: Some(proto::User {
                id: result.user.id,
                username: result.user.username,
                email: result.user.email,
            }),
        }))
    }

    async fn login(
        &self,
        request: Request<proto::LoginRequest>,
    ) -> Result<Response<proto::AuthResponse>, Status> {
        let request = request.into_inner();

        let result = self
            .auth_service
            .login(DomainLoginRequest {
                username: request.username,
                password: request.password,
            })
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(proto::AuthResponse {
            access_token: result.token,
            user: Some(proto::User {
                id: result.user.id,
                username: result.user.username,
                email: result.user.email,
            }),
        }))
    }

    async fn get_post(
        &self,
        request: Request<proto::GetPostRequest>,
    ) -> Result<Response<proto::PostResponse>, Status> {
        let post = self
            .blog_service
            .get_post(request.into_inner().id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(proto::PostResponse {
            post: Some(post_to_proto(post)),
        }))
    }

    async fn list_posts(
        &self,
        request: Request<proto::ListPostsRequest>,
    ) -> Result<Response<ListPostsResponse>, Status> {
        let request = request.into_inner();

        let page = if request.page == 0 { 1 } else { request.page };
        let page_size = if request.page_size == 0 {
            10
        } else {
            request.page_size
        };

        if page < 1 || !(1..=100).contains(&page_size) {
            return Err(Status::invalid_argument("Invalid pagination parameters"));
        }

        let offset = i64::from(page - 1) * i64::from(page_size);

        let (posts, total) = self
            .blog_service
            .list_posts(i64::from(page_size), offset)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(proto::ListPostsResponse {
            posts: posts.into_iter().map(post_to_proto).collect(),
            total,
        }))
    }

    async fn create_post(
        &self,
        request: Request<proto::CreatePostRequest>,
    ) -> Result<Response<proto::PostResponse>, Status> {
        let user_id = self.authenticated_user(&request)?;

        let data = request.into_inner();

        let post = self
            .blog_service
            .create_post(
                user_id,
                crate::domain::post::CreatePostRequest {
                    title: data.title,
                    content: data.content,
                },
            )
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(proto::PostResponse {
            post: Some(post_to_proto(post)),
        }))
    }

    async fn update_post(
        &self,
        request: Request<proto::UpdatePostRequest>,
    ) -> Result<Response<proto::PostResponse>, Status> {
        let user_id = self.authenticated_user(&request)?;

        let data = request.into_inner();

        let post = self
            .blog_service
            .update_post(
                data.id,
                user_id,
                crate::domain::post::UpdatePostRequest {
                    title: data.title,
                    content: data.content,
                },
            )
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(proto::PostResponse {
            post: Some(post_to_proto(post)),
        }))
    }

    async fn delete_post(
        &self,
        request: Request<proto::DeletePostRequest>,
    ) -> Result<Response<proto::DeletePostResponse>, Status> {
        let user_id = self.authenticated_user(&request)?;

        let data = request.into_inner();

        self.blog_service
            .delete_post(data.id, user_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(proto::DeletePostResponse::default()))
    }
}

fn post_to_proto(post: Post) -> proto::Post {
    proto::Post {
        id: post.id,
        title: post.title,
        content: post.content,
        author_id: post.author_id,
        created_at: post.created_at.to_rfc3339(),
        updated_at: post.updated_at.to_rfc3339(),
    }
}

fn domain_error_to_status(error: DomainError) -> Status {
    match error {
        DomainError::PostNotFound => Status::not_found(error.to_string()),
        DomainError::UserAlreadyExists => Status::already_exists(error.to_string()),
        DomainError::InvalidCredentials => Status::unauthenticated(error.to_string()),
        DomainError::Forbidden => Status::permission_denied(error.to_string()),
        DomainError::InvalidInput(message) => Status::invalid_argument(message),
        DomainError::Internal => Status::internal("Internal server error"),
    }
}
