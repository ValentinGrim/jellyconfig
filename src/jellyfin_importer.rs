use nix::unistd::getuid;
use procfs::process::all_processes;
use std::fs::File;
use std::io;
use std::path::Path;
use tar::Archive;

pub struct JellyfinImporter;

impl JellyfinImporter {
    /// Check requirement before srtating to import...
    pub fn check_requirements() -> io::Result<()> {
        if !getuid().is_root() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Root privileges required. Please run with sudo.",
            ));
        }

        // Check if jellyfin process is running
        let processes =
            all_processes().map_err(|e| io::Error::other(format!("Failed to read /proc: {e}")))?;

        let is_running = processes.flatten().any(|p| {
            p.stat()
                .map(|s| s.comm.to_lowercase().contains("jellyfin"))
                .unwrap_or(false)
        });

        if is_running {
            return Err(io::Error::other(
                "Jellyfin is still running. Stop the service before importing (e.g., sudo systemctl stop jellyfin).",
            ));
        }

        Ok(())
    }

    /// Import the given jexport to the current system
    pub fn import(input_path: &Path) -> io::Result<()> {
        Self::check_requirements()?;

        let file = File::open(input_path)?;
        let decoder = zstd::stream::read::Decoder::new(file)?;
        let mut archive = Archive::new(decoder);
        archive.unpack("/")?;

        Ok(())
    }
}
