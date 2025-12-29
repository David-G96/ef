use tokio::time::Instant;
use color_eyre::Result;

#[derive(Debug, Clone, PartialEq)]
pub struct FpsCounter {
    last_tick_update: Instant,
    tick_count: u32,
    ticks_per_second: f64,

    last_frame_update: Instant,
    frame_count: u32,
    frames_per_second: f64,
}

impl FpsCounter {
    pub fn new() -> Self {
        Self {
            last_tick_update: Instant::now(),
            tick_count: 0,
            ticks_per_second: 0.0,
            last_frame_update: Instant::now(),
            frame_count: 0,
            frames_per_second: 0.0,
        }
    }

    fn app_tick(&mut self) -> Result<()> {
        self.tick_count += 1;
        let now = Instant::now();
        let elapsed = (now - self.last_tick_update).as_secs_f64();
        if elapsed >= 1.0 {
            self.ticks_per_second = self.tick_count as f64 / elapsed;
            self.last_tick_update = now;
            self.tick_count = 0;
        }
        Ok(())
    }

    fn render_tick(&mut self) -> Result<()> {
        self.frame_count += 1;
        let now = Instant::now();
        let elapsed = (now - self.last_frame_update).as_secs_f64();
        if elapsed >= 1.0 {
            self.frames_per_second = self.frame_count as f64 / elapsed;
            self.last_frame_update = now;
            self.frame_count = 0;
        }
        Ok(())
    }
}