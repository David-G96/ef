#[derive(Debug)]
pub struct Config {
    /// 帧率
    frame_rate: f64,
    /// tick率，没啥用
    /// 如果以后有用的话估计就是光标的闪烁速度了
    tick_rate: f64,
    /// 监听文件的帧率，别设太高
    watch_rate: f64,
}

impl Config {
    pub fn frame_rate(&self) -> f64 {
        self.frame_rate
    }

    pub fn tick_rate(&self) -> f64 {
        self.tick_rate
    }

    pub fn watch_rate(&self) -> f64 {
        self.watch_rate
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frame_rate: 60.0,
            tick_rate: 5.0,
            watch_rate: 1.0,
        }
    }
}
