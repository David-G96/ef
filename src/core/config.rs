use color_eyre::{
    Result as Res,
    eyre::{Ok, bail},
};
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::{fs::File, path::PathBuf};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// 帧率，应该默认为60,最高可配置144左右？
    pub frame_rate: f64,
    /// tick率
    /// 如果以后有用的话估计就是光标的闪烁速度了
    pub tick_rate: f64,
    // // 监听文件的帧率，别设太高
    // // 更正：监听不需要帧率！
    // pub watch_rate: f64,
    pub default_path: Option<PathBuf>,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse_from_str(str: &str) -> Res<Self> {
        toml::from_str(str).map_err(|e| e.into())
    }

    pub fn parse() -> Res<Self> {
        if let Some(base_dir) = BaseDirs::new() {
            let config_dir = base_dir.config_dir();
            config_dir.to_path_buf().push("config.toml");

            toml::from_str("").map_err(|e| e.into())
        } else {
            bail!("failed to get base dir!")
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frame_rate: 60.0,
            tick_rate: 4.0,
            default_path: None,
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_config_parse() {
        let config_file = r#""#;
    }
}
