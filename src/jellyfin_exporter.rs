use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use tar::Builder;

pub struct JellyfinExporter {
    pub configs: Vec<PathBuf>,
    pub databases: Vec<PathBuf>,
}

impl Default for JellyfinExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl JellyfinExporter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            configs: Vec::new(),
            databases: Vec::new(),
        }
    }

    /// Scans standard Linux paths to find Jellyfin configuration and database files.
    /// This method ignores binary files, cache, logs, and sensitive session keys.
    pub fn scan(&mut self) {
        let search_paths = [
            "/etc/jellyfin",
            "/var/lib/jellyfin",
            "/usr/share/jellyfin/web",
        ];

        for path in search_paths.iter().map(Path::new).filter(|p| p.exists()) {
            self.scan_recursive(path);
        }
    }

    /// Recursively explores a directory and find needed files.
    fn scan_recursive(&mut self, dir: &Path) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if path.is_dir() {
                // Ignore temporary folders, logs, and security keys to keep the export clean
                let to_ignore = [
                    "cache",
                    "log",
                    "metadata",
                    "transcoding-temp",
                    "DataProtection-Keys",
                ];

                // Skip plugin versioned folders (e.g., PluginName_1.0.0.0)
                let is_plugin_bin =
                    filename.contains('_') && path.to_string_lossy().contains("/plugins/");

                if !to_ignore.contains(&filename) && !is_plugin_bin {
                    self.scan_recursive(&path);
                }
            } else {
                let path_str = path.to_string_lossy();

                // Store db
                if std::path::Path::new(filename)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("db"))
                {
                    self.databases.push(path);
                }
                // Store config
                else if std::path::Path::new(filename)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("xml"))
                    || std::path::Path::new(filename)
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
                {
                    let needed = [
                        "system.xml",
                        "network.xml",
                        "branding.xml",
                        "livetv.xml",
                        "encoding.xml",
                        "database.xml",
                        "config.json",
                        "settings.json",
                        "options.xml",
                    ];

                    // Exclude XMLs/JSONs found in bin or driver directories
                    let is_binary_related = path_str.contains("/bin/")
                        || path_str.contains("vulkan")
                        || path_str.contains("drirc.d");
                    let is_plugin_conf = path_str.contains("plugins/configurations");

                    if (needed.contains(&filename) || is_plugin_conf) && !is_binary_related {
                        self.configs.push(path);
                    }
                }
            }
        }
    }

    pub fn export(&self, export_path: &Path) -> io::Result<()> {
        let file = File::create(export_path)?;
        let encoder = zstd::stream::Encoder::new(file, 3)?;
        let mut tar = Builder::new(encoder);

        for path in self.databases.iter().chain(self.configs.iter()) {
            if path.exists() {
                tar.append_path_with_name(path, path.strip_prefix("/").unwrap_or(path))?;
            }
        }

        let encoder = tar.into_inner()?;
        encoder.finish()?;
        Ok(())
    }
}
