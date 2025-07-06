use std::fs;
use std::fs::read_to_string;
use std::io;
use std::io::Write;

#[derive(Debug)]
pub struct GimConfig {
    pub default_add_as_desc: bool,
    pub verbose: bool,
    pub editor: String,
}

impl Default for GimConfig {
    fn default() -> Self {
        Self {
            default_add_as_desc: false, // Default behavior: add as title, not description
            verbose: false,             // Default behavior: no labels, clean display
            editor: "vim".to_string(),  // Default editor
        }
    }
}

pub fn get_config_path() -> io::Result<std::path::PathBuf> {
    let home = match std::env::var("HOME") {
        Ok(home) => std::path::PathBuf::from(home),
        Err(_) => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Could not find HOME directory",
            ))
        }
    };

    let config_dir = home.join(".config").join("gim");

    // Create config directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    Ok(config_dir.join("config.toml"))
}

pub fn load_config() -> GimConfig {
    match get_config_path() {
        Ok(config_path) => {
            if let Ok(content) = read_to_string(&config_path) {
                parse_config(&content)
            } else {
                GimConfig::default()
            }
        }
        Err(_) => GimConfig::default(),
    }
}

pub fn parse_config(content: &str) -> GimConfig {
    let mut config = GimConfig::default();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"').trim_matches('\'');

            match key {
                "default_add_as_desc" => {
                    config.default_add_as_desc = value.parse().unwrap_or(false);
                }
                "verbose" => {
                    config.verbose = value.parse().unwrap_or(false);
                }
                "editor" => {
                    config.editor = value.to_string();
                }
                _ => {} // Ignore unknown keys
            }
        }
    }

    config
}

pub fn save_config(config: &GimConfig) -> io::Result<()> {
    let config_path = get_config_path()?;
    let content = format!(
        r#"# Gim Configuration File
# Edit this file to customize gim behavior
# Location: ~/.config/gim/config.toml
# Use 'gim config edit' to edit this file

# ============================================================================
# COMMIT MESSAGE BEHAVIOR
# ============================================================================

# Controls default behavior of 'gim add' command
# 
# When true:  'gim add "message"' adds to description (below title)
# When false: 'gim add "message"' adds to title (comma-separated)
# 
# You can always override with:
#   'gim add --desc "description"' to add to description
#   'gim add "title"' to add to title (default behavior)
#
# Default: false
default_add_as_desc = {}

# ============================================================================
# DISPLAY FORMATTING
# ============================================================================

# Controls status display format
#
# When true:  Shows labels and structure
#             Current commit:
#                 Title: feat/feature-name, task1, task2
#                 Description:
#                     Detailed description here
#
# When false: Clean format without labels
#             Current commit:
#                 feat/feature-name, task1, task2    (bold)
#                 Detailed description here          (italic)
#
# Default: false 
verbose = {}

# ============================================================================
# EDITOR SETTINGS
# ============================================================================

# Default editor for 'gim edit' and 'gim config edit' commands
#
# This should be the command name or full path to your preferred text editor.
# The editor will be launched with the file path as an argument.
#
# Default: "vim"
editor = "{}"
"#,
        config.default_add_as_desc, config.verbose, config.editor
    );

    let mut file = fs::File::create(config_path)?;
    write!(file, "{content}")?;
    Ok(())
}