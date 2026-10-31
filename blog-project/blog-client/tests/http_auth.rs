use blog_client::{BlogClient, Transport};

#[tokio::test]
async fn test_http_auth() {
    let mut client = BlogClient::new(Transport::Http("http://127.0.0.1:8080".to_string()))
        .await
        .expect("Failed to create HTTP client");

    let username = format!(
        "test_client_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    let email = format!("{username}@example.com");

    let result = client
        .register(&username, &email, "TestPassword123!")
        .await
        .expect("Registration failed");

    assert_eq!(result.user.username, username);
    assert!(client.get_token().is_some());

    let result = client
        .login(&username, "TestPassword123!")
        .await
        .expect("Login failed");

    assert_eq!(result.user.username, username);
    assert_eq!(client.get_token(), Some(result.token.as_str()));
}
