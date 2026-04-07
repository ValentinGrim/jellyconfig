pub mod jellyfin_exporter;
pub mod jellyfin_importer;

use chrono::Local;
use clap::{Parser, Subcommand};
use jellyfin_exporter::JellyfinExporter;
use jellyfin_importer::JellyfinImporter;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Export configuration to a .jexport file
    Export {
        /// Output file path (default: export_YYYY-MM-DD_HH-MM-SS.jexport)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Import configuration from a .jexport file (Requires Sudo & Stopped Service)
    Import {
        /// Input file path
        input: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Export { output } => {
            let mut final_output = if let Some(path) = output {
                path
            } else {
                let now = Local::now();
                let timestamp = now.format("%Y-%m-%d_%H-%M-%S").to_string();
                PathBuf::from(format!("export_{timestamp}.jexport"))
            };

            if final_output.extension().and_then(|s| s.to_str()) != Some("jexport") {
                final_output.set_extension("jexport");
            }

            let mut exporter = JellyfinExporter::new();
            println!("🔍 Scanning system for Jellyfin files...");
            exporter.scan();

            println!(
                "✅ Scan complete: {} databases and {} config files found.",
                exporter.databases.len(),
                exporter.configs.len()
            );

            if let Err(e) = exporter.export(&final_output) {
                eprintln!("❌ Export failed: {e}");
                std::process::exit(1);
            }
            println!("🎉 Successfully exported to: {}", final_output.display());
        }

        Commands::Import { input } => {
            if !input.exists() {
                eprintln!("❌ Error: Input file '{}' not found.", input.display());
                std::process::exit(1);
            }

            if let Err(e) = JellyfinImporter::import(&input) {
                eprintln!("❌ Import blocked: {e}");
                std::process::exit(1);
            }
            println!("✅ Restoration completed successfully.");
        }
    }
}
