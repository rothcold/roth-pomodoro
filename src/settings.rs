#[derive(Debug, Clone, Copy)]
pub enum Screen {
    Timer,
    Settings,
}

#[derive(Debug, Clone, Copy)]
pub struct Settings {
    pub work_seconds: u32,
    pub short_break_seconds: u32,
    pub long_break_seconds: u32,
    pub long_break_every: u32,
}

impl Settings {
    pub const DEFAULT_LONG_BREAK_EVERY: u32 = 4;
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            work_seconds: super::WORK_LENGTH,
            short_break_seconds: super::BREAK_LENGTH,
            long_break_seconds: super::LONG_BREAK_LENGTH,
            long_break_every: Self::DEFAULT_LONG_BREAK_EVERY,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SettingsDraft {
    pub work_minutes: String,
    pub short_break_minutes: String,
    pub long_break_minutes: String,
    pub long_break_every: String,
}

impl SettingsDraft {
    pub fn from_settings(settings: Settings) -> Self {
        Self {
            work_minutes: (settings.work_seconds / 60).to_string(),
            short_break_minutes: (settings.short_break_seconds / 60).to_string(),
            long_break_minutes: (settings.long_break_seconds / 60).to_string(),
            long_break_every: settings.long_break_every.to_string(),
        }
    }

    pub fn parse(&self) -> Option<Settings> {
        let work_minutes: u32 = self.work_minutes.trim().parse().ok()?;
        let short_break_minutes: u32 = self.short_break_minutes.trim().parse().ok()?;
        let long_break_minutes: u32 = self.long_break_minutes.trim().parse().ok()?;
        let long_break_every: u32 = self.long_break_every.trim().parse().ok()?;

        if work_minutes == 0
            || short_break_minutes == 0
            || long_break_minutes == 0
            || long_break_every == 0
        {
            return None;
        }

        Some(Settings {
            work_seconds: work_minutes.saturating_mul(60),
            short_break_seconds: short_break_minutes.saturating_mul(60),
            long_break_seconds: long_break_minutes.saturating_mul(60),
            long_break_every,
        })
    }
}
