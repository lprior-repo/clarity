#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use clap::{Parser, Subcommand};
use clarity_web::domain::ScenarioField;
use clarity_web::server::{
    calculate_quality_server, extract_fields_server, get_ai_provider_status_server,
    validate_hole_punching_server, validate_straw_man_traps_server, validate_vorp,
};
use clarity_web::types::Answer;

#[derive(Parser)]
#[command(name = "clarity", version, about = "Clarity - Double Diamond Planning CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Extract structured fields from freeform text
    Extract {
        /// Input text to extract from
        #[arg(short, long)]
        input: String,
    },
    /// Calculate quality score from answers
    Quality {
        /// Answers as JSON array
        #[arg(short, long)]
        answers: String,
    },
    /// Validate a persona against straw man traps
    ValidateStrawMan {
        /// Persona description text
        #[arg(short, long)]
        persona: String,
    },
    /// Validate VORP (Value, Obvious, Real, Possible)
    ValidateVorp {
        #[arg(long)]
        value: String,
        #[arg(long)]
        obvious: String,
        #[arg(long)]
        real: String,
        #[arg(long)]
        possible: String,
    },
    /// Validate hole punching for a scenario
    ValidateHoles {
        #[arg(long)]
        discovery: Option<String>,
        #[arg(long)]
        edge_case: Option<String>,
        #[arg(long)]
        motivation: Option<String>,
    },
    /// Show AI provider status
    Status,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt::Subscriber::default()
    ).ok();

    let cli = Cli::parse();

    match cli.command {
        Commands::Extract { input } => {
            let result = extract_fields_server(input, None).await?;
            let json = serde_json::to_string_pretty(&result)?;
            println!("{json}");
        }
        Commands::Quality { answers } => {
            let parsed: Vec<Answer> = serde_json::from_str(&answers)?;
            let result = calculate_quality_server(parsed, None, None).await?;
            let json = serde_json::to_string_pretty(&result)?;
            println!("{json}");
        }
        Commands::ValidateStrawMan { persona } => {
            let result = validate_straw_man_traps_server(persona, None).await?;
            let json = serde_json::to_string_pretty(&result)?;
            println!("{json}");
        }
        Commands::ValidateVorp {
            value,
            obvious,
            real,
            possible,
        } => {
            let result = validate_vorp(value, obvious, real, possible, None).await?;
            let json = serde_json::to_string_pretty(&result)?;
            println!("{json}");
        }
        Commands::ValidateHoles {
            discovery,
            edge_case,
            motivation,
        } => {
            let scenario = ScenarioField::new(
                discovery.map_or_else(String::new, |s| s),
                edge_case.map_or_else(String::new, |s| s),
                motivation.map_or_else(String::new, |s| s),
            );
            let result = validate_hole_punching_server(scenario, None).await?;
            let json = serde_json::to_string_pretty(&result)?;
            println!("{json}");
        }
        Commands::Status => {
            let result = get_ai_provider_status_server().await?;
            let json = serde_json::to_string_pretty(&result)?;
            println!("{json}");
        }
    }

    Ok(())
}
