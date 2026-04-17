mod api;
mod ui;
mod score;
mod export;
mod config;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gitpulse")]
#[command(version, about = "GitHub profile analytics TUI with Developer Health Score")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// GitHub username to analyze
    #[arg(short, long, env = "GITHUB_USERNAME")]
    username: Option<String>,

    /// GitHub personal access token (increases rate limit)
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch the interactive TUI dashboard
    Dashboard {
        /// GitHub username to analyze
        username: String,
    },
    /// Export analysis report
    Export {
        /// GitHub username to analyze
        username: String,
        /// Output format: json, md
        #[arg(short, long, default_value = "json")]
        format: String,
        /// Output file path (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Generate a score badge URL
    Badge {
        /// GitHub username to analyze
        username: String,
    },
    /// Show quick stats (no TUI)
    Stats {
        /// GitHub username to analyze
        username: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let token = cli.token.or_else(|| config::load_token());

    match cli.command {
        Some(Commands::Dashboard { username }) => {
            ui::run_dashboard(&username, token).await?;
        }
        Some(Commands::Export { username, format, output }) => {
            export::run_export(&username, token, &format, output).await?;
        }
        Some(Commands::Badge { username }) => {
            export::generate_badge(&username, token).await?;
        }
        Some(Commands::Stats { username }) => {
            let client = api::GithubClient::new(token);
            let data = client.fetch_all(&username).await?;
            let scorer = score::Scorer::new();
            let report = scorer.compute(&data);
            println!("{}", report.summary_text());
        }
        None => {
            let username = cli.username.unwrap_or_else(|| {
                eprintln!("Error: provide a username with -u or run `gitpulse dashboard <USERNAME>`");
                std::process::exit(1);
            });
            ui::run_dashboard(&username, token).await?;
        }
    }

    Ok(())
}
