use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    /// 帧率，应该默认为60,最高可配置144左右？
    pub frame_rate: f64,
    /// tick率
    /// 如果以后有用的话估计就是光标的闪烁速度了
    pub tick_rate: f64,
    // // 监听文件的帧率，别设太高
    // // 更正：监听不需要帧率！
    // pub watch_rate: f64,
    pub default_path: PathBuf,
}

impl Config {}

impl Default for Config {
    fn default() -> Self {
        Self {
            frame_rate: 60.0,
            tick_rate: 4.0,
            // watch_rate: 1.0,
            default_path: PathBuf::default(),
        }
    }
}
