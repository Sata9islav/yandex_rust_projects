use blog_client::{BlogClient, Transport};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test]
async fn test_grpc_auth() {
    let mut client = BlogClient::new(Transport::Grpc("http://127.0.0.1:50051".to_string()))
        .await
        .expect("Failed to create gRPC client");

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let username = format!("grpc_user_{timestamp}");
    let email = format!("{username}@example.com");

    let registered = client
        .register(&username, &email, "TestPassword123!")
        .await
        .expect("gRPC registration failed");

    assert_eq!(registered.user.username, username);
    assert_eq!(client.get_token(), Some(registered.token.as_str()));

    let logged_in = client
        .login(&username, "TestPassword123!")
        .await
        .expect("gRPC login failed");

    assert_eq!(logged_in.user.id, registered.user.id);
    assert_eq!(client.get_token(), Some(logged_in.token.as_str()));
}
