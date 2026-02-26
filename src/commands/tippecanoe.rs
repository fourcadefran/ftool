use std::path::Path;
use std::process::Command;

/// Configuration for a tippecanoe tile generation run.
#[derive(Debug, Clone)]
pub struct TippecanoeConfig {
    pub min_zoom: u8,
    pub max_zoom: u8,
    pub no_feature_limit: bool,
    pub no_tile_size_limit: bool,
    pub drop_densest_as_needed: bool,
}

impl Default for TippecanoeConfig {
    fn default() -> Self {
        Self {
            min_zoom: 0,
            max_zoom: 14,
            no_feature_limit: false,
            no_tile_size_limit: false,
            drop_densest_as_needed: false,
        }
    }
}

/// Named presets that configure zoom ranges for common data types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Preset {
    /// No preset applied — use whatever values are already set on the config.
    Custom,
    /// General-purpose vector tiles (zoom 6–18).
    Generic,
    /// Parcel / polygon data (zoom 0–16).
    Parcels,
    /// Point data (zoom 0–18).
    Points,
}

impl Preset {
    pub fn label(self) -> &'static str {
        match self {
            Preset::Custom => "Custom",
            Preset::Generic => "Generic (6-18)",
            Preset::Parcels => "Parcels (0-16)",
            Preset::Points => "Points (0-18)",
        }
    }
}

impl TippecanoeConfig {
    /// Apply a named preset, overwriting `min_zoom` and `max_zoom`.
    /// `Preset::Custom` is a no-op.
    pub fn apply_preset(&mut self, preset: Preset) {
        match preset {
            Preset::Custom => {}
            Preset::Generic => {
                self.min_zoom = 6;
                self.max_zoom = 18;
            }
            Preset::Parcels => {
                self.min_zoom = 0;
                self.max_zoom = 16;
            }
            Preset::Points => {
                self.min_zoom = 0;
                self.max_zoom = 18;
            }
        }
    }
}

/// Returns `true` when `tippecanoe` is found on `PATH` and responds to
/// `--version` with a successful exit code.
pub fn check_tippecanoe_installed() -> bool {
    Command::new("tippecanoe")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Build and execute a `tippecanoe` command for `input`, writing the output
/// `.pmtiles` file next to the input file.
///
/// Returns `Ok(output_path)` on success, or `Err(stderr)` on failure.
pub fn run_tippecanoe(input: &Path, config: &TippecanoeConfig) -> Result<String, String> {
    // Derive the output path: same directory + stem + ".pmtiles"
    let stem = input
        .file_stem()
        .ok_or_else(|| "Input path has no file stem".to_string())?
        .to_string_lossy();

    let parent = input
        .parent()
        .unwrap_or_else(|| Path::new("."));

    let output_path = parent.join(format!("{}.pmtiles", stem));
    let output_str = output_path
        .to_str()
        .ok_or_else(|| "Output path contains invalid UTF-8".to_string())?
        .to_string();

    // Build the command
    let mut cmd = Command::new("tippecanoe");

    cmd.arg("--force")
        .arg("--read-parallel")
        .arg(format!("--minimum-zoom={}", config.min_zoom))
        .arg(format!("--maximum-zoom={}", config.max_zoom))
        .arg(format!("--output={}", output_str));

    if config.no_feature_limit {
        cmd.arg("--no-feature-limit");
    }

    if config.no_tile_size_limit {
        cmd.arg("--no-tile-size-limit");
    }

    if config.drop_densest_as_needed {
        cmd.arg("--drop-densest-as-needed");
    }

    // Input file is always last
    cmd.arg(input);

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to spawn tippecanoe: {}", e))?;

    if output.status.success() {
        Ok(output_str)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        Err(stderr)
    }
}
