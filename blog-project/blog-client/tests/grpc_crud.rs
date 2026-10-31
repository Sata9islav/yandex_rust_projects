use std::time::{SystemTime, UNIX_EPOCH};

use blog_client::{BlogClient, Transport, error::BlogClientError};

#[tokio::test]
async fn test_grpc_crud() {
    let mut client = BlogClient::new(Transport::Grpc("http://127.0.0.1:50051".to_string()))
        .await
        .expect("Failed to create gRPC client");

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let username = format!("grpc_crud_user_{timestamp}");
    let email = format!("{username}@example.com");

    let auth = client
        .register(&username, &email, "TestPassword123!")
        .await
        .expect("Registration failed");

    assert!(client.get_token().is_some());

    // 2. Создание поста
    let post = client
        .create_post("Original gRPC title", "Original gRPC content")
        .await
        .expect("Create post failed");

    assert_eq!(post.title, "Original gRPC title");
    assert_eq!(post.author_id, auth.user.id);

    let post_id = post.id;

    let fetched = client.get_post(post_id).await.expect("Get post failed");

    assert_eq!(fetched.id, post_id);
    assert_eq!(fetched.content, "Original gRPC content");

    let updated = client
        .update_post(post_id, "Updated gRPC title", "Updated gRPC content")
        .await
        .expect("Update post failed");

    assert_eq!(updated.title, "Updated gRPC title");
    assert_eq!(updated.content, "Updated gRPC content");
    assert_eq!(updated.author_id, auth.user.id);

    let list = client.list_posts(10, 0).await.expect("List posts failed");

    assert!(list.total >= 1);
    assert_eq!(list.limit, 10);
    assert_eq!(list.offset, 0);

    client
        .delete_post(post_id)
        .await
        .expect("Delete post failed");

    let result = client.get_post(post_id).await;

    assert!(matches!(result, Err(BlogClientError::NotFound)));
}
