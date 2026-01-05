use iced::window;

mod db;
mod pomodoro_timer;
mod settings;

use pomodoro_timer::PomodoroTimer;

pub mod audio;
// const WORK_LENGTH: u32 = 15;
// const BREAK_LENGTH: u32 = 3;
// const LONG_BREAK_LENGTH: u32 = 9;

pub const WORK_LENGTH: u32 = 1500;
pub const BREAK_LENGTH: u32 = 300;
pub const LONG_BREAK_LENGTH: u32 = 900;

fn main() -> iced::Result {
    // Add a logo for this app
    iced::application(
        PomodoroTimer::new,
        PomodoroTimer::update,
        PomodoroTimer::view,
    )
    .title("Pomodoro Timer")
    .subscription(PomodoroTimer::subscription)
    .theme(iced::Theme::CatppuccinLatte)
    .window(window::Settings {
        size: iced::Size::new(600.0, 500.0),
        resizable: true,
        level: window::Level::Normal,
        icon: Some(
            window::icon::from_file(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/images/icon.png"
            ))
            .expect("icon file should be reachable and in ICO file format"),
        ),
        ..Default::default()
    })
    .run()
}
