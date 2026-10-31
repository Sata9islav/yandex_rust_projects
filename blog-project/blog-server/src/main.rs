mod application;
mod data;
mod domain;
mod infrastructure;
mod presentation;

use std::{env, net::SocketAddr, sync::Arc};

use actix_cors::Cors;
use actix_web::{App, HttpServer, guard, http::Method, web};
use actix_web_httpauth::middleware::HttpAuthentication;

use blog_proto::blog::blog_service_server::BlogServiceServer;

use tonic::transport::Server;
use tracing::info;
use tracing_subscriber::EnvFilter;

use data::{post_repository::PostRepository, user_repository::UserRepository};

use infrastructure::{
    database::{create_pool, run_migrations},
    jwt::JwtService,
};

use crate::application::{auth_service::AuthService, blog_service::BlogService};

use crate::presentation::{
    grpc_service::BlogGrpcService, http_handlers, middleware::jwt_validator,
};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::from_path(concat!(env!("CARGO_MANIFEST_DIR"), "/.env"))?;

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let pool = create_pool(&database_url).await?;

    run_migrations(&pool).await?;
    info!("Miggrations apllied successfully!");

    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET is not set");
    let jwt_service = Arc::new(JwtService::new(&jwt_secret)?);

    let auth_service = Arc::new(AuthService::new(
        UserRepository::new(pool.clone()),
        Arc::clone(&jwt_service),
    ));

    let blog_service = Arc::new(BlogService::new(PostRepository::new(pool.clone())));

    let grpc_service = BlogGrpcService::new(
        Arc::clone(&auth_service),
        Arc::clone(&blog_service),
        Arc::clone(&jwt_service),
    );

    let grpc_addr: SocketAddr = "127.0.0.1:50051".parse()?;

    let grpc_server = Server::builder()
        .add_service(BlogServiceServer::new(grpc_service))
        .serve(grpc_addr);

    let http_auth = web::Data::new(Arc::clone(&auth_service));
    let http_blog = web::Data::new(Arc::clone(&blog_service));
    let http_jwt = web::Data::new(Arc::clone(&jwt_service));

    let http_server = HttpServer::new(move || {
        App::new()
            .app_data(http_auth.clone())
            .app_data(http_blog.clone())
            .app_data(http_jwt.clone())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .service(
                web::scope("/api/auth")
                    .route("/register", web::post().to(http_handlers::register))
                    .route("/login", web::post().to(http_handlers::login)),
            )
            .service(
                web::resource("/api/posts")
                    .guard(guard::Post())
                    .wrap(HttpAuthentication::bearer(jwt_validator))
                    .route(web::post().to(http_handlers::create_post)),
            )
            .service(
                web::resource("/api/posts/{id}")
                    .guard(guard::fn_guard(|ctx| {
                        matches!(ctx.head().method, Method::PUT | Method::DELETE)
                    }))
                    .wrap(HttpAuthentication::bearer(jwt_validator))
                    .route(web::put().to(http_handlers::update_post))
                    .route(web::delete().to(http_handlers::delete_post)),
            )
            .service(
                web::resource("/api/posts")
                    .guard(guard::Get())
                    .route(web::get().to(http_handlers::list_posts)),
            )
            .service(
                web::resource("/api/posts/{id}")
                    .guard(guard::Get())
                    .route(web::get().to(http_handlers::get_post)),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run();

    info!("HTTP listening on 127.0.0.1:8080");
    info!("gRPC listening on {}", grpc_addr);

    tokio::try_join!(
        async {
            http_server.await?;
            Ok::<(), anyhow::Error>(())
        },
        async {
            grpc_server.await?;
            Ok::<(), anyhow::Error>(())
        }
    )?;

    Ok(())
}
