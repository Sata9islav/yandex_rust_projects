use blog_client::BlogClient;

use crate::{cli::Commands, token};

pub async fn execute(
    client: &mut BlogClient,
    command: Commands,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Register {
            username,
            email,
            password,
        } => {
            let auth = client.register(&username, &email, &password).await?;

            token::save_token(&auth.token)?;

            println!("Registration successful!");
            println!("User ID: {}", auth.user.id);
            println!("Username: {}", auth.user.username);
            println!("Email: {}", auth.user.email);
            println!("JWT saved to {}", token::token_path()?.display());
        }

        Commands::Login { username, password } => {
            let auth = client.login(&username, &password).await?;

            token::save_token(&auth.token)?;

            println!("Login successful!");
            println!("User ID: {}", auth.user.id);
            println!("Username: {}", auth.user.username);
            println!("JWT saved to {}", token::token_path()?.display());
        }

        Commands::Create { title, content } => {
            let post = client.create_post(&title, &content).await?;

            println!("Post created successfully!");
            print_post(&post);
        }

        Commands::Get { id } => {
            let post = client.get_post(id).await?;
            print_post(&post);
        }

        Commands::Update { id, title, content } => {
            if title.is_none() && content.is_none() {
                return Err("Specify --title or --content".into());
            }

            let current = client.get_post(id).await?;

            let title = title.unwrap_or(current.title);
            let content = content.unwrap_or(current.content);

            let post = client.update_post(id, &title, &content).await?;

            println!("Post updated successfully!");
            print_post(&post);
        }

        Commands::Delete { id } => {
            client.delete_post(id).await?;
            println!("Post {id} deleted successfully!");
        }

        Commands::List { limit, offset } => {
            let result = client.list_posts(limit, offset).await?;

            println!("Total posts: {}", result.total);
            println!("Limit: {}, Offset: {}", result.limit, result.offset);

            if result.posts.is_empty() {
                println!("No posts found");
            } else {
                for post in &result.posts {
                    println!();
                    print_post(post);
                }
            }
        }
    }

    Ok(())
}

fn print_post(post: &blog_client::models::Post) {
    println!("---------------------------");
    println!("ID: {}", post.id);
    println!("Title: {}", post.title);
    println!("Content: {}", post.content);
    println!("Author ID: {}", post.author_id);
    println!("Created: {}", post.created_at);
    println!("Updated: {}", post.updated_at);
    println!("---------------------------");
}
