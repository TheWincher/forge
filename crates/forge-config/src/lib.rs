use std::{env, fs, path::Path};

use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub workspace_root: Option<std::path::PathBuf>,
}

impl Config {
    pub fn load() -> Config {
        let user_path = dirs::config_dir().map(|d| d.join("forge").join("config.toml"));
        let project_path = env::current_dir().ok().map(|d| d.join("forge.toml"));

        Config::load_merged(user_path.as_deref(), project_path.as_deref())
    }

    fn load_merged(user_path: Option<&Path>, project_path: Option<&Path>) -> Config {
        let user_config = user_path.map(Config::load_one).unwrap_or_default();
        let project_config = project_path.map(Config::load_one).unwrap_or_default();

        Config::merge(user_config, project_config)
    }

    fn load_one(path: &Path) -> Config {
        if let Ok(contents) = fs::read_to_string(path) {
            match Config::parse(contents.as_str()) {
                Ok(config) => config,
                Err(err) => {
                    tracing::warn!(path = %path.display(), %err);
                    Config::default()
                }
            }
        } else {
            Config::default()
        }
    }

    fn parse(contents: &str) -> Result<Config, toml::de::Error> {
        toml::from_str(contents)
    }

    fn merge(base: Config, override_: Config) -> Config {
        Config {
            workspace_root: override_.workspace_root.or(base.workspace_root),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn write_temp_file(name: &str, contents: &str) -> PathBuf {
        let path =
            env::temp_dir().join(format!("forge-config-test-{}-{}", std::process::id(), name));
        fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn parse_valid_toml() {
        let config = Config::parse(r#"workspace_root = "/tmp/foo""#).unwrap();
        assert_eq!(config.workspace_root, Some(PathBuf::from("/tmp/foo")));
    }

    #[test]
    fn parse_invalid_toml_is_err() {
        assert!(Config::parse("not = [valid").is_err());
    }

    #[test]
    fn merge_project_overrides_user() {
        let base = Config {
            workspace_root: Some(PathBuf::from("/user")),
        };
        let override_ = Config {
            workspace_root: Some(PathBuf::from("/project")),
        };

        let merged = Config::merge(base, override_);

        assert_eq!(merged.workspace_root, Some(PathBuf::from("/project")));
    }

    #[test]
    fn merge_keeps_user_when_project_omits() {
        let base = Config {
            workspace_root: Some(PathBuf::from("/user")),
        };
        let override_ = Config::default();

        let merged = Config::merge(base, override_);

        assert_eq!(merged.workspace_root, Some(PathBuf::from("/user")));
    }

    #[test]
    fn load_one_missing_file_returns_default() {
        let path = env::temp_dir().join("forge-config-test-does-not-exist.toml");
        let config = Config::load_one(&path);
        assert_eq!(config.workspace_root, None);
    }

    #[test]
    fn load_one_invalid_toml_returns_default() {
        let path = write_temp_file("invalid.toml", "not = [valid");

        let config = Config::load_one(&path);

        assert_eq!(config.workspace_root, None);
        fs::remove_file(&path).ok();
    }

    #[test]
    fn load_merged_project_overrides_user_end_to_end() {
        let user_path = write_temp_file("user.toml", r#"workspace_root = "/user""#);
        let project_path = write_temp_file("project.toml", r#"workspace_root = "/project""#);

        let config = Config::load_merged(Some(&user_path), Some(&project_path));

        assert_eq!(config.workspace_root, Some(PathBuf::from("/project")));
        fs::remove_file(&user_path).ok();
        fs::remove_file(&project_path).ok();
    }

    #[test]
    fn load_merged_none_paths_returns_default() {
        let config = Config::load_merged(None, None);
        assert_eq!(config.workspace_root, None);
    }
}
