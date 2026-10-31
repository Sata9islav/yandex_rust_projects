use blog_client::{BlogClient, Transport};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test]
async fn test_http_crud() {
    let mut client = BlogClient::new(Transport::Http("http://127.0.0.1:8080".to_string()))
        .await
        .expect("Failed to create client");

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let username = format!("crud_user_{timestamp}");
    let email = format!("{username}@example.com");

    let auth = client
        .register(&username, &email, "TestPassword123!")
        .await
        .expect("Registration failed");

    assert!(client.get_token().is_some());

    let post = client
        .create_post("Original title", "Original content")
        .await
        .expect("Create post failed");

    assert_eq!(post.title, "Original title");
    assert_eq!(post.author_id, auth.user.id);

    let post_id = post.id;

    let fetched = client.get_post(post_id).await.expect("Get post failed");

    assert_eq!(fetched.id, post_id);

    let updated = client
        .update_post(post_id, "Updated title", "Updated content")
        .await
        .expect("Update post failed");

    assert_eq!(updated.title, "Updated title");
    assert_eq!(updated.content, "Updated content");

    let list = client.list_posts(100, 0).await.expect("List posts failed");

    assert!(list.posts.iter().any(|p| p.id == post_id));

    client
        .delete_post(post_id)
        .await
        .expect("Delete post failed");

    let result = client.get_post(post_id).await;

    assert!(matches!(
        result,
        Err(blog_client::error::BlogClientError::NotFound)
    ));
}
