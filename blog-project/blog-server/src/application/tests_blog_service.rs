use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    application::blog_service::BlogService,
    data::post_repository::PostRepository,
    domain::{
        error::DomainError,
        post::{CreatePostRequest, UpdatePostRequest},
    },
};

#[tokio::test]
async fn test_blog_service() -> anyhow::Result<()> {
    let env_path = concat!(env!("CARGO_MANIFEST_DIR"), "/.env");
    dotenvy::from_path(env_path)?;

    let database_url = std::env::var("DATABASE_URL")?;
    let pool = sqlx::PgPool::connect(&database_url).await?;

    let repository = PostRepository::new(pool.clone());
    let service = BlogService::new(repository);

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();

    let alice_name = format!("alice_{timestamp}");
    let bob_name = format!("bob_{timestamp}");

    let alice_id: i64 = sqlx::query_scalar(
        r#"INSERT INTO users (username, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(&alice_name)
    .bind(format!("{alice_name}@example.com"))
    .bind("test_hash")
    .fetch_one(&pool)
    .await?;

    let bob_id: i64 = sqlx::query_scalar(
        r#" INSERT INTO users (username, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(&bob_name)
    .bind(format!("{bob_name}@example.com"))
    .bind("test_hash")
    .fetch_one(&pool)
    .await?;

    let post = service
        .create_post(
            alice_id,
            CreatePostRequest {
                title: "First_post".into(),
                content: "Example".into(),
            },
        )
        .await?;
    assert_eq!(post.author_id, alice_id);

    let found = service.get_post(post.id).await?;
    assert_eq!(found.id, post.id);
    assert_eq!(found.title, "First_post");

    let forbidden_update = service
        .update_post(
            post.id,
            bob_id,
            UpdatePostRequest {
                title: "Forbidden post's title".into(),
                content: "Forbidden post's content".into(),
            },
        )
        .await;
    assert!(matches!(forbidden_update, Err(DomainError::Forbidden)));

    let unchanged = service.get_post(post.id).await?;
    assert_eq!(unchanged.title, "First_post");

    let forbidden_delete = service.delete_post(post.id, bob_id).await;
    assert!(matches!(forbidden_delete, Err(DomainError::Forbidden)));
    assert!(service.get_post(post.id).await.is_ok());

    let updated = service
        .update_post(
            post.id,
            alice_id,
            UpdatePostRequest {
                title: "Updated title".into(),
                content: "Updated content".into(),
            },
        )
        .await?;
    assert_eq!(updated.title, "Updated title");

    let (posts, total) = service.list_posts(100, 0).await?;
    assert!(posts.iter().any(|p| p.id == post.id));
    assert!(total >= 1);

    let invalid_page = service.list_posts(0, 0).await;

    assert!(matches!(invalid_page, Err(DomainError::InvalidInput(_))));

    service.delete_post(post.id, alice_id).await?;

    let missing = service.get_post(post.id).await;
    assert!(matches!(missing, Err(DomainError::PostNotFound)));

    sqlx::query("DELETE FROM users WHERE id = $1 OR id = $2")
        .bind(alice_id)
        .bind(bob_id)
        .execute(&pool)
        .await?;

    pool.close().await;

    Ok(())
}
