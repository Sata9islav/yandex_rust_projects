use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "blog-cli")]
#[command(about = "CLI client for Blog API")]
#[command(version)]
pub struct Cli {
    #[arg(long, global = true)]
    pub grpc: bool,

    #[arg(long, global = true)]
    pub server: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Register {
        #[arg(long)]
        username: String,

        #[arg(long)]
        email: String,

        #[arg(long)]
        password: String,
    },

    Login {
        #[arg(long)]
        username: String,

        #[arg(long)]
        password: String,
    },

    Create {
        #[arg(long)]
        title: String,

        #[arg(long)]
        content: String,
    },

    Get {
        #[arg(long, value_parser = clap::value_parser!(i64).range(1..))]
        id: i64,
    },

    Update {
        #[arg(long, value_parser = clap::value_parser!(i64).range(1..))]
        id: i64,

        #[arg(long)]
        title: Option<String>,

        #[arg(long)]
        content: Option<String>,
    },

    Delete {
        #[arg(long, value_parser = clap::value_parser!(i64).range(1..))]
        id: i64,
    },

    List {
        #[arg(long, default_value_t = 20, value_parser = clap::value_parser!(i64).range(1..=100))]
        limit: i64,

        #[arg(long, default_value_t = 0, value_parser = clap::value_parser!(i64).range(0..))]
        offset: i64,
    },
}
