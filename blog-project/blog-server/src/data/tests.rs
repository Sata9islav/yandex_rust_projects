use std::time::{SystemTime, UNIX_EPOCH};

use crate::data::{post_repository::PostRepository, user_repository::UserRepository};

#[tokio::test]
async fn test_repositories() -> anyhow::Result<()> {
    let env_path = concat!(env!("CARGO_MANIFEST_DIR"), "/.env");
    dotenvy::from_path(env_path)?;

    let database_url = std::env::var("DATABASE_URL")?;
    let pool = sqlx::PgPool::connect(&database_url).await?;

    let user_repo = UserRepository::new(pool.clone());
    let post_repo = PostRepository::new(pool.clone());

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();

    let username = format!("test_{timestamp}");
    let email = format!("{username}@example.com");

    let user = user_repo.create(&username, &email, "test_hash").await?;

    assert_eq!(user.username, username);

    let found_by_id = user_repo
        .find_by_username(&username)
        .await?
        .expect("User not found");

    assert_eq!(found_by_id.email, email);

    let post = post_repo
        .create("Test title", "Test content", user.id)
        .await?;

    let found_post = post_repo
        .find_by_id(post.id)
        .await?
        .expect("Post not found");
    assert_eq!(found_post.title, "Test title");

    let updated = post_repo
        .update(post.id, user.id, "Updated title", "Updated content")
        .await?
        .expect("Post not found for update");
    assert_eq!(updated.title, "Updated title");

    let posts = post_repo.list(100, 0).await?;
    assert!(posts.iter().any(|p| p.id == post.id));

    let count = post_repo.count().await?;
    assert!(count >= 1);

    let deleted = post_repo.delete(post.id, user.id).await?;
    assert!(deleted);

    assert!(post_repo.find_by_id(post.id).await?.is_none());

    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user.id)
        .execute(&pool)
        .await?;

    pool.close().await;
    Ok(())
}
