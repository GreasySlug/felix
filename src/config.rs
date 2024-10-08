use crate::errors::FxError;

use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

pub const FELIX: &str = "felix";
const CONFIG_FILE: &str = "config.yaml";
const CONFIG_FILE_ANOTHER_EXT: &str = "config.yml";

#[derive(Debug, Clone)]
pub struct ConfigWithPath {
    pub config_path: Option<PathBuf>,
    pub config: Config,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub default: Option<String>,
    pub match_vim_exit_behavior: Option<bool>,
    pub exec: Option<BTreeMap<String, Vec<String>>>,
    pub ignore_case: Option<bool>,
    pub color: Option<ConfigColor>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct ConfigColor {
    pub dir_fg: Colorname,
    pub file_fg: Colorname,
    pub symlink_fg: Colorname,
    pub dirty_fg: Colorname,
}

impl Default for ConfigColor {
    fn default() -> Self {
        Self {
            dir_fg: Colorname::LightCyan,
            file_fg: Colorname::LightWhite,
            symlink_fg: Colorname::LightYellow,
            dirty_fg: Colorname::Red,
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum Colorname {
    Black,        // 0
    Red,          // 1
    Green,        // 2
    Yellow,       // 3
    Blue,         // 4
    Magenta,      // 5
    Cyan,         // 6
    White,        // 7
    LightBlack,   // 8
    LightRed,     // 9
    LightGreen,   // 10
    LightYellow,  // 11
    LightBlue,    // 12
    LightMagenta, // 13
    LightCyan,    // 14
    LightWhite,   // 15
    Rgb(u8, u8, u8),
    AnsiValue(u8),
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default: Default::default(),
            match_vim_exit_behavior: Default::default(),
            exec: Default::default(),
            ignore_case: Some(false),
            color: Some(Default::default()),
        }
    }
}

pub fn read_config(p: &Path) -> Result<ConfigWithPath, FxError> {
    let s = read_to_string(p)?;
    let deserialized: Config = serde_yaml::from_str(&s)?;
    Ok(ConfigWithPath {
        config_path: Some(p.to_path_buf()),
        config: deserialized,
    })
}

pub fn read_config_or_default() -> Result<ConfigWithPath, FxError> {
    //First, declare default config file path.
    let (config_file_path1, config_file_path2) = {
        let mut config_path = {
            let mut path = dirs::config_dir()
                .ok_or_else(|| FxError::Dirs("Cannot read the config directory.".to_string()))?;
            path.push(FELIX);
            path
        };
        let mut another = config_path.clone();
        config_path.push(CONFIG_FILE);
        another.push(CONFIG_FILE_ANOTHER_EXT);
        (config_path, another)
    };

    //On macOS, felix looks for 2 paths:
    //First `$HOME/Library/Application Support/felix/config.yaml(yml)`,
    //and if it fails,
    //`$HOME/.config/felix/config.yaml(yml)`.
    let config_file_paths = if cfg!(target_os = "macos") {
        let (alt_config_file_path1, alt_config_file_path2) = {
            let mut config_path = dirs::home_dir()
                .ok_or_else(|| FxError::Dirs("Cannot read the home directory.".to_string()))?;
            config_path.push(".config");
            config_path.push("FELIX");
            let mut another = config_path.clone();
            config_path.push(CONFIG_FILE);
            another.push(CONFIG_FILE_ANOTHER_EXT);
            (config_path, another)
        };
        vec![
            config_file_path1,
            config_file_path2,
            alt_config_file_path1,
            alt_config_file_path2,
        ]
    } else {
        vec![config_file_path1, config_file_path2]
    };

    let mut config_file: Option<PathBuf> = None;
    for p in config_file_paths {
        if p.exists() {
            config_file = Some(p);
            break;
        }
    }

    if let Some(config_file) = config_file {
        read_config(&config_file)
    } else {
        Ok(ConfigWithPath {
            config_path: None,
            config: Config::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_default_config() {
        let default_config: Config = serde_yaml::from_str("").unwrap();
        assert_eq!(default_config.default, None);
        assert_eq!(default_config.match_vim_exit_behavior, None);
        assert_eq!(default_config.exec, None);
        assert_eq!(default_config.ignore_case, None);
        assert_eq!(default_config.color, None);
    }

    #[test]
    fn test_read_full_config() {
        let full_config: Config = serde_yaml::from_str(
            r#"
default: nvim
match_vim_exit_behavior: true
exec:
  zathura:
    [pdf]
  'feh -.':
    [jpg, jpeg, png, gif, svg, hdr]
ignore_case: true
color:
  dir_fg: LightCyan
  file_fg: LightWhite
  symlink_fg: LightYellow
  dirty_fg: Red
"#,
        )
        .unwrap();
        assert_eq!(full_config.default, Some("nvim".to_string()));
        assert_eq!(full_config.match_vim_exit_behavior, Some(true));
        assert_eq!(
            full_config.exec.clone().unwrap().get("zathura"),
            Some(&vec!["pdf".to_string()])
        );
        assert_eq!(
            full_config.exec.unwrap().get("feh -."),
            Some(&vec![
                "jpg".to_string(),
                "jpeg".to_string(),
                "png".to_string(),
                "gif".to_string(),
                "svg".to_string(),
                "hdr".to_string()
            ])
        );
        assert_eq!(full_config.ignore_case, Some(true));
        assert_eq!(
            full_config.color.clone().unwrap().dir_fg,
            Colorname::LightCyan
        );
        assert_eq!(
            full_config.color.clone().unwrap().file_fg,
            Colorname::LightWhite
        );
        assert_eq!(
            full_config.color.clone().unwrap().symlink_fg,
            Colorname::LightYellow
        );
        assert_eq!(full_config.color.unwrap().dirty_fg, Colorname::Red);
    }
}
