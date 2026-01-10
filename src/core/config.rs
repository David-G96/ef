use color_eyre::Result as Res;
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Config {
    /// 帧率，默认为60
    pub frame_rate: f64,
    /// tick率,默认为4
    /// 如果以后有用的话估计就是光标的闪烁速度了
    pub tick_rate: f64,
    /// 默认打开路径，默认为空，也就是当前目录wd
    pub default_path: Option<PathBuf>,
    /// 是否显示隐藏文件
    pub show_hidden: bool,
    /// 是否尊重 gitignore
    pub respect_gitignore: bool,
}

#[derive(Debug)]
pub enum ConfigStatus {
    /// 从配置文件成功加载
    Loaded(Config),
    /// 配置文件不存在或无法获取目录，回退到默认值
    Default(Config),
}

impl ConfigStatus {
    pub fn config(self) -> Config {
        match self {
            ConfigStatus::Loaded(c) | ConfigStatus::Default(c) => c,
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn parse_from_str(str: &str) -> Res<Self> {
        toml::from_str(str).map_err(|e| e.into())
    }

    /// 新增：从指定路径解析配置，方便测试
    pub fn parse_from_path(path: impl AsRef<Path>) -> Res<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::parse_from_str(&content)
    }

    /// parse from the default config path
    /// Err if config content error.
    pub fn parse() -> Res<ConfigStatus> {
        let config_path =
            BaseDirs::new().map(|base| base.config_dir().join("ef").join("config.toml"));

        if let Some(path) = config_path.as_ref().filter(|p| p.exists()) {
            return Ok(ConfigStatus::Loaded(Self::parse_from_path(path)?));
        }

        Ok(ConfigStatus::Default(Self::default()))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frame_rate: 60.0,
            tick_rate: 4.0,
            default_path: None,
            show_hidden: false,
            respect_gitignore: true,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_config_parse() {
        let config_file = r#"frame_rate = 60
        tick_rate = 4"#;

        let config: Config = toml::from_str(config_file).expect("failed to parse config content");
        let expected = Config::new();

        assert_eq!(expected, config);
    }

    #[test]
    fn test_parse_from_path() -> Res<()> {
        // 使用 tempfile 创建一个临时文件
        let temp_file = tempfile::NamedTempFile::new()?;
        let path = temp_file.path();

        // 写入测试内容
        let content = r#"frame_rate = 120.0
        tick_rate = 10.0"#;
        std::fs::write(path, content)?;

        // 测试解析逻辑
        let config = Config::parse_from_path(path)?;
        assert_eq!(config.frame_rate, 120.0);
        assert_eq!(config.tick_rate, 10.0);
        Ok(())
    }
}
