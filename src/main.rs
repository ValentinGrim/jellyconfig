pub mod jellyfin_exporter;
pub mod jellyfin_importer;
pub mod utils;

use chrono::Local;
use clap::{Parser, Subcommand};
use jellyfin_exporter::JellyfinExporter;
use jellyfin_importer::JellyfinImporter;
use std::path::PathBuf;
use utils::pb_with_text;

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
            let mut export_path = if let Some(path) = output {
                path
            } else {
                // If user didn't input file name, generate one based on current datetime
                let now = Local::now();
                let timestamp = now.format("%Y-%m-%d_%H-%M-%S").to_string();
                PathBuf::from(format!("export_{timestamp}.jexport"))
            };

            // Add ext if not present
            if export_path.extension().and_then(|s| s.to_str()) != Some("jexport") {
                export_path.set_extension("jexport");
            }

            let mut exporter = JellyfinExporter::new();
            let pb = pb_with_text("🔍 Scanning system for Jellyfin files...");
            exporter.scan();

            pb.finish_with_message(format!(
                "✅ Scan complete: {} databases and {} config files found.",
                exporter.databases.len(),
                exporter.configs.len()
            ));

            let pb = pb_with_text("📦 Archiving files...");
            if let Err(e) = exporter.export(&export_path) {
                pb.finish_with_message(format!("❌ Export failed: {e}"));
                std::process::exit(1);
            }
            pb.finish_with_message(format!(
                "🎉 Successfully exported to: {}",
                export_path.display()
            ));
        }

        Commands::Import { input } => {
            if !input.exists() {
                eprintln!("❌ Error: Input file '{}' not found.", input.display());
                std::process::exit(1);
            }

            let pb = pb_with_text(format!("📂 Restoring files from {}...", input.display()));
            if let Err(e) = JellyfinImporter::import(&input) {
                pb.finish_with_message(format!("❌ Import blocked: {e}"));
                std::process::exit(1);
            }
            pb.finish_with_message("✅ Restoration completed successfully.");
        }
    }
}
