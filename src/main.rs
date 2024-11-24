use iced::{
    time,
    widget::{button, container, row, text, Column},
    window,
    Alignment::Center,
    Element, Length, Subscription,
};
use rodio::{self, Source};
use std::fs::File;
use std::{
    io::BufReader,
    time::{Duration, Instant},
};

const WORK_LENGTH: u32 = 1500;
const BREAK_LENGTH: u32 = 300;
const LONG_BREAK_LENGTH: u32 = 900;
struct PomodoroTimer {
    time_left: u32, // Time left in seconds
    end_time: Option<Instant>,
    work_periods: u32,
    is_running: bool,
    started: bool,
    is_work_period: bool, // true for work, false for break
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Tick(Instant),
    StartStop,
    Reset,
}

impl PomodoroTimer {
    pub fn new() -> PomodoroTimer {
        PomodoroTimer {
            time_left: WORK_LENGTH,
            end_time: None,
            work_periods: 0,
            is_running: false,
            started: false,
            is_work_period: true,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let start_stop_button = button(if self.is_running {
            "Pause"
        } else if self.started {
            "Resume"
        } else {
            "Start"
        });
        let reset_button = button("Reset");
        let buttons = row![
            start_stop_button.on_press(Message::StartStop),
            reset_button.on_press(Message::Reset)
        ]
        .spacing(10);
        let column = Column::new()
            .align_x(Center)
            .push(
                text(format!(
                    "{:02}:{:02}",
                    self.time_left / 60,
                    self.time_left % 60
                ))
                .size(50),
            )
            .push(buttons);
        container(column).center(Length::Fill).into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        match self.is_running {
            true => time::every(Duration::from_millis(100)).map(Message::Tick),
            false => Subscription::none(),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Tick(now) => {
                if self.is_running && self.time_left > 0 {
                    self.time_left = (self.end_time.unwrap() - now).as_secs() as u32;
                }
                if self.time_left == 0 {
                    self.started = false;
                    // Increment work periods when a work period ends
                    if self.is_work_period {
                        self.work_periods += 1;
                    }

                    // Toggle between work and break
                    self.is_work_period = !self.is_work_period;

                    // Set the time left based on the current period
                    self.time_left = if self.is_work_period {
                        WORK_LENGTH
                    // Long break every 4 work periods
                    } else if self.work_periods % 4 == 0 {
                        LONG_BREAK_LENGTH
                    } else {
                        BREAK_LENGTH
                    };
                    self.is_running = false;
                    // Play a sound
                    if let Err(err) = rodio::OutputStream::try_default() {
                        println!("Error initializing sound: {}", err);
                    } else {
                        let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
                        let file = BufReader::new(
                            File::open(concat!(
                                env!("CARGO_MANIFEST_DIR"),
                                "/assets/audio/alarm.mp3"
                            ))
                            .unwrap(),
                        );
                        let source = rodio::Decoder::new(file).unwrap();
                        let stream_handle_clone = stream_handle.clone();
                        std::thread::spawn(move || {
                            let _ = stream_handle_clone.play_raw(source.convert_samples());
                            std::thread::sleep(std::time::Duration::from_secs(6));
                        });
                    }
                }
            }
            Message::StartStop => {
                self.is_running = !self.is_running;
                if self.is_running {
                    self.started = true;
                    self.end_time =
                        Some(Instant::now() + Duration::from_secs(self.time_left as u64));
                }
            }
            Message::Reset => {
                self.is_running = false;
                self.is_work_period = true;
                self.time_left = WORK_LENGTH;
                self.started = false;
                self.end_time = None;
                self.work_periods = 0;
            }
        }
    }

    pub fn theme(&self) -> iced::Theme {
        iced::Theme::CatppuccinLatte
    }
}

impl Default for PomodoroTimer {
    fn default() -> Self {
        Self::new()
    }
}

fn main() -> iced::Result {
    // Add a logo for this app
    iced::application("Pomodoro Timer", PomodoroTimer::update, PomodoroTimer::view)
        .subscription(PomodoroTimer::subscription)
        .theme(PomodoroTimer::theme)
        .window(window::Settings {
            resizable: false,
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
