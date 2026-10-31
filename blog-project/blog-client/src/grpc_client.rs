use chrono::{DateTime, Utc};
use tonic::metadata::MetadataValue;

use crate::models::{Post, PostList};

use blog_proto::blog::{self as proto, blog_service_client::BlogServiceClient};

use tonic::{Request, transport::Channel};

use crate::{
    error::BlogClientError,
    models::{AuthResponse, User},
};

type GrpcClient = BlogServiceClient<Channel>;

pub async fn register(
    client: &mut GrpcClient,
    username: &str,
    email: &str,
    password: &str,
) -> Result<AuthResponse, BlogClientError> {
    let request = Request::new(proto::RegisterRequest {
        username: username.to_string(),
        email: email.to_string(),
        password: password.to_string(),
    });

    let response = client.register(request).await?.into_inner();

    auth_from_proto(response)
}

pub async fn login(
    client: &mut GrpcClient,
    username: &str,
    password: &str,
) -> Result<AuthResponse, BlogClientError> {
    let request = Request::new(proto::LoginRequest {
        username: username.to_string(),
        password: password.to_string(),
    });

    let response = client.login(request).await?.into_inner();

    auth_from_proto(response)
}

fn auth_from_proto(response: proto::AuthResponse) -> Result<AuthResponse, BlogClientError> {
    let user = response.user.ok_or_else(|| {
        BlogClientError::UnexpectedResponse(
            "Missing user in gRPC authentication response".to_string(),
        )
    })?;

    Ok(AuthResponse {
        token: response.access_token,
        user: User {
            id: user.id,
            username: user.username,
            email: user.email,
        },
    })
}

pub async fn create_post(
    client: &mut GrpcClient,
    token: &str,
    title: &str,
    content: &str,
) -> Result<Post, BlogClientError> {
    let mut request = Request::new(proto::CreatePostRequest {
        title: title.to_string(),
        content: content.to_string(),
    });

    add_token(&mut request, token)?;

    let response = client.create_post(request).await?.into_inner();

    post_from_response(response)
}

pub async fn get_post(client: &mut GrpcClient, id: i64) -> Result<Post, BlogClientError> {
    let request = Request::new(proto::GetPostRequest { id });

    let response = client.get_post(request).await?.into_inner();

    post_from_response(response)
}

pub async fn update_post(
    client: &mut GrpcClient,
    token: &str,
    id: i64,
    title: &str,
    content: &str,
) -> Result<Post, BlogClientError> {
    let mut request = Request::new(proto::UpdatePostRequest {
        id,
        title: title.to_string(),
        content: content.to_string(),
    });

    add_token(&mut request, token)?;

    let response = client.update_post(request).await?.into_inner();

    post_from_response(response)
}

pub async fn delete_post(
    client: &mut GrpcClient,
    token: &str,
    id: i64,
) -> Result<(), BlogClientError> {
    let mut request = Request::new(proto::DeletePostRequest { id });

    add_token(&mut request, token)?;

    client.delete_post(request).await?;

    Ok(())
}

pub async fn list_posts(
    client: &mut GrpcClient,
    limit: i64,
    offset: i64,
) -> Result<PostList, BlogClientError> {
    if !(1..=100).contains(&limit) || offset < 0 {
        return Err(BlogClientError::InvalidRequest(
            "Invalid pagination parameters".into(),
        ));
    }

    // Постраничная gRPC-пагинация не поддерживает
    // произвольный offset, поэтому проверяем кратность.
    if offset % limit != 0 {
        return Err(BlogClientError::InvalidRequest(
            "For gRPC, offset must be divisible by limit".into(),
        ));
    }

    let page = i32::try_from(offset / limit + 1)
        .map_err(|_| BlogClientError::InvalidRequest("Page number is too large".into()))?;

    let request = Request::new(proto::ListPostsRequest {
        page,
        page_size: limit as i32,
    });

    let response = client.list_posts(request).await?.into_inner();

    let posts = response
        .posts
        .into_iter()
        .map(post_from_proto)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(PostList {
        posts,
        total: response.total,
        limit,
        offset,
    })
}

fn add_token<T>(request: &mut Request<T>, token: &str) -> Result<(), BlogClientError> {
    let value = format!("Bearer {token}");

    let metadata = MetadataValue::try_from(value.as_str())
        .map_err(|_| BlogClientError::InvalidRequest("Invalid JWT metadata".into()))?;

    request.metadata_mut().insert("authorization", metadata);

    Ok(())
}

fn post_from_response(response: proto::PostResponse) -> Result<Post, BlogClientError> {
    let post = response.post.ok_or_else(|| {
        BlogClientError::UnexpectedResponse("Missing post in gRPC response".into())
    })?;

    post_from_proto(post)
}

fn post_from_proto(post: proto::Post) -> Result<Post, BlogClientError> {
    let parse_date = |value: &str| -> Result<DateTime<Utc>, BlogClientError> {
        DateTime::parse_from_rfc3339(value)
            .map(|date| date.with_timezone(&Utc))
            .map_err(|error| {
                BlogClientError::UnexpectedResponse(format!("Invalid post timestamp: {error}"))
            })
    };

    Ok(Post {
        id: post.id,
        title: post.title,
        content: post.content,
        author_id: post.author_id,
        created_at: parse_date(&post.created_at)?,
        updated_at: parse_date(&post.updated_at)?,
    })
}
