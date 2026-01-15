use crate::settings::{Screen, Settings, SettingsDraft};
use iced::{
    Alignment::Center,
    Background, Border, Color, Element, Length, Subscription, Theme, time,
    widget::{Column, button, container, row, text, text_input, tooltip},
};
use rodio::{Sink, Source};
use std::{
    sync::mpsc::{self, Sender},
    thread,
    time::{Duration, Instant},
};

pub struct PomodoroTimer {
    time_left: u32,
    end_time: Option<Instant>,
    work_periods: u32,
    completed_pomodoros: u32,
    is_running: bool,
    started: bool,
    is_work_period: bool,
    audio_sender: Sender<AudioCommand>,
    screen: Screen,
    settings: Settings,
    settings_draft: SettingsDraft,
    settings_error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick(Instant),
    StartStop,
    Reset,
    ResetPomoCounter,
    OpenSettings,
    CloseSettings,
    SettingsWorkMinutesChanged(String),
    SettingsShortBreakMinutesChanged(String),
    SettingsLongBreakMinutesChanged(String),
    SettingsLongBreakEveryChanged(String),
    SaveSettings,
}

#[derive(Debug, Clone)]
enum AudioCommand {
    Alarm,
    Stop,
}

impl PomodoroTimer {
    pub fn new() -> PomodoroTimer {
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
            let sink = rodio::Sink::try_new(&stream_handle).unwrap();

            loop {
                if let Ok(command) = receiver.try_recv() {
                    process_audio_command(command, &sink);
                }
                thread::sleep(std::time::Duration::from_millis(100));
            }
        });

        let settings = crate::db::load_settings();
        let completed_pomodoros = crate::db::load_completed_pomodoros();

        PomodoroTimer {
            time_left: settings.work_seconds,
            end_time: None,
            work_periods: 0,
            completed_pomodoros,
            is_running: false,
            started: false,
            is_work_period: true,
            audio_sender: sender,
            screen: Screen::Timer,
            settings,
            settings_draft: SettingsDraft::from_settings(settings),
            settings_error: None,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        match self.screen {
            Screen::Timer => self.view_timer(),
            Screen::Settings => self.view_settings(),
        }
    }

    fn view_timer(&self) -> Element<'_, Message> {
        // Determine current period type and color
        let (period_text, period_color) = if self.is_work_period {
            ("🍅 Work Time", [1.0, 0.42, 0.42]) // Tomato red
        } else if self.work_periods % self.settings.long_break_every == 0 {
            ("☕ Long Break", [0.58, 0.88, 0.83]) // Teal
        } else {
            ("☕ Short Break", [0.31, 0.80, 0.77]) // Light blue
        };

        // Progress indicator
        let current_cycle = (self.work_periods % self.settings.long_break_every) + 1;
        let progress_text = if self.is_work_period {
            format!(
                "Pomodoro {}/{} until long break",
                current_cycle, self.settings.long_break_every
            )
        } else {
            "Break time - relax!".to_string()
        };

        // Top-right utility buttons (icon-only with tooltips)
        let reset_button = tooltip(
            button(text("↻").size(20))
                .padding(10)
                .style(transparent_button_style)
                .on_press(Message::Reset),
            "Reset",
            tooltip::Position::Bottom,
        );

        let reset_counter_button = tooltip(
            button(text("⟲").size(20))
                .padding(10)
                .style(transparent_button_style)
                .on_press(Message::ResetPomoCounter),
            "Reset Count",
            tooltip::Position::Bottom,
        );

        let settings_button = tooltip(
            button(text("⚙").size(20))
                .padding(10)
                .style(transparent_button_style)
                .on_press(Message::OpenSettings),
            "Settings",
            tooltip::Position::Bottom,
        );

        let top_right_buttons =
            row![reset_button, reset_counter_button, settings_button].spacing(10);

        // Top bar with buttons aligned to the right
        let top_bar = row![
            container(text("")).width(Length::Fill), // Spacer to push buttons right
            top_right_buttons
        ]
        .padding(10)
        .width(Length::Fill);

        // Period type header
        let period_header = text(period_text).size(32).color(period_color);

        // Large timer display
        let timer_display = text(format!(
            "{:02}:{:02}",
            self.time_left / 60,
            self.time_left % 60
        ))
        .size(100)
        .color(period_color);

        // Progress and completed count
        let progress_info = Column::new()
            .align_x(Center)
            .spacing(5)
            .push(text(progress_text).size(16))
            .push(text(format!("✓ Completed: {}", self.completed_pomodoros)).size(18));

        // Large centered start/stop button
        let start_stop_button = button(
            text(if self.is_running {
                "⏸ Pause"
            } else if self.started {
                "▶ Resume"
            } else {
                "▶ Start"
            })
            .size(28),
        )
        .padding([20, 40])
        .style(transparent_button_style)
        .on_press(Message::StartStop);

        // Center content column
        let center_content = Column::new()
            .align_x(Center)
            .spacing(30)
            .push(period_header)
            .push(timer_display)
            .push(progress_info)
            .push(text("").size(20)) // Spacer
            .push(start_stop_button);

        // Main column with top bar and centered content
        let main_column = Column::new().push(top_bar).push(
            container(center_content)
                .center(Length::Fill)
                .height(Length::Fill),
        );

        container(main_column)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_settings(&self) -> Element<'_, Message> {
        // Settings header
        let header = text("⚙ Settings").size(40);

        // Form fields with improved layout
        let work = Column::new()
            .spacing(8)
            .push(text("🍅 Work Duration (minutes)").size(16))
            .push(
                text_input("25", &self.settings_draft.work_minutes)
                    .on_input(Message::SettingsWorkMinutesChanged)
                    .padding(12)
                    .size(16),
            );

        let short_break = Column::new()
            .spacing(8)
            .push(text("☕ Short Break (minutes)").size(16))
            .push(
                text_input("5", &self.settings_draft.short_break_minutes)
                    .on_input(Message::SettingsShortBreakMinutesChanged)
                    .padding(12)
                    .size(16),
            );

        let long_break = Column::new()
            .spacing(8)
            .push(text("☕ Long Break (minutes)").size(16))
            .push(
                text_input("15", &self.settings_draft.long_break_minutes)
                    .on_input(Message::SettingsLongBreakMinutesChanged)
                    .padding(12)
                    .size(16),
            );

        let long_every = Column::new()
            .spacing(8)
            .push(text("🔄 Long Break Every (pomodoros)").size(16))
            .push(
                text_input("4", &self.settings_draft.long_break_every)
                    .on_input(Message::SettingsLongBreakEveryChanged)
                    .padding(12)
                    .size(16),
            );

        // Action buttons with distinct styling
        let actions = row![
            button(text("✓ Save").size(18))
                .style(transparent_button_style)
                .on_press(Message::SaveSettings)
                .padding([12, 24]),
            button(text("✕ Cancel").size(18))
                .style(transparent_button_style)
                .on_press(Message::CloseSettings)
                .padding([12, 24])
        ]
        .spacing(15);

        // Build main column
        let mut column = Column::new()
            .align_x(Center)
            .spacing(20)
            .padding(40)
            .push(header)
            .push(text("").size(5)) // Spacer
            .push(work)
            .push(short_break)
            .push(long_break)
            .push(long_every);

        // Error message with red color
        if let Some(error) = &self.settings_error {
            column = column.push(text(format!("⚠ {}", error)).size(16).color([1.0, 0.3, 0.3]));
        }

        column = column
            .push(text("").size(5)) // Spacer
            .push(actions);

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
                    if self.is_work_period {
                        self.work_periods += 1;
                        self.completed_pomodoros = self.completed_pomodoros.saturating_add(1);
                        crate::db::save_completed_pomodoros(self.completed_pomodoros);
                    }

                    self.is_work_period = !self.is_work_period;

                    self.time_left = if self.is_work_period {
                        self.settings.work_seconds
                    } else if self.work_periods % self.settings.long_break_every == 0 {
                        self.settings.long_break_seconds
                    } else {
                        self.settings.short_break_seconds
                    };
                    self.is_running = false;

                    if let Err(err) = rodio::OutputStream::try_default() {
                        println!("Error initializing sound: {}", err);
                    } else {
                        self.audio_sender
                            .send(AudioCommand::Alarm)
                            .expect("Could not send audio command");
                    }
                }
            }
            Message::StartStop => {
                self.is_running = !self.is_running;
                if self.is_running {
                    self.audio_sender
                        .send(AudioCommand::Stop)
                        .expect("Could not send stop command");
                    self.started = true;
                    self.end_time =
                        Some(Instant::now() + Duration::from_secs(self.time_left as u64));
                }
            }
            Message::Reset => {
                self.audio_sender
                    .send(AudioCommand::Stop)
                    .expect("Could not send stop command");
                self.is_running = false;
                self.is_work_period = true;
                self.time_left = self.settings.work_seconds;
                self.started = false;
                self.end_time = None;
                self.work_periods = 0;
            }
            Message::ResetPomoCounter => {
                self.completed_pomodoros = 0;
                crate::db::save_completed_pomodoros(self.completed_pomodoros);
            }
            Message::OpenSettings => {
                self.is_running = false;
                self.end_time = None;
                self.settings_error = None;
                self.settings_draft = SettingsDraft::from_settings(self.settings);
                self.screen = Screen::Settings;
            }
            Message::CloseSettings => {
                self.settings_error = None;
                self.screen = Screen::Timer;
            }
            Message::SettingsWorkMinutesChanged(value) => {
                self.settings_draft.work_minutes = value;
            }
            Message::SettingsShortBreakMinutesChanged(value) => {
                self.settings_draft.short_break_minutes = value;
            }
            Message::SettingsLongBreakMinutesChanged(value) => {
                self.settings_draft.long_break_minutes = value;
            }
            Message::SettingsLongBreakEveryChanged(value) => {
                self.settings_draft.long_break_every = value;
            }
            Message::SaveSettings => {
                if let Some(settings) = self.settings_draft.parse() {
                    self.settings = settings;
                    crate::db::save_settings(self.settings);
                    self.settings_error = None;

                    self.audio_sender
                        .send(AudioCommand::Stop)
                        .expect("Could not send stop command");
                    self.is_running = false;
                    self.is_work_period = true;
                    self.time_left = self.settings.work_seconds;
                    self.started = false;
                    self.end_time = None;
                    self.work_periods = 0;

                    self.screen = Screen::Timer;
                } else {
                    self.settings_error = Some(
                        "Invalid settings. Use positive numbers for minutes and pomos.".to_string(),
                    );
                }
            }
        }
    }
}

impl Default for PomodoroTimer {
    fn default() -> Self {
        Self::new()
    }
}

fn transparent_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let base_style = button::Style {
        background: Some(Background::Color(Color::from_rgba(0.024, 0.58, 0.58, 1.0))),
        border: Border {
            color: Color::from_rgba(0.024, 0.58, 0.58, 1.0),
            width: 0.0,
            radius: 4.0.into(),
        },
        text_color: Color::from_rgb(0.3, 0.3, 0.3),
        ..Default::default()
    };

    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgba(0.024, 0.48, 0.48, 1.0))),
            ..base_style
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color::from_rgba(0.024, 0.42, 0.42, 1.0))),
            ..base_style
        },
        _ => base_style,
    }
}

fn process_audio_command(command: AudioCommand, sink: &Sink) {
    match command {
        AudioCommand::Alarm => {
            let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
            let source = rodio::source::SineWave::new(240.0)
                .take_duration(Duration::from_millis(500))
                .amplify(0.20);
            stream_handle.play_raw(source.convert_samples()).unwrap();
            std::thread::sleep(Duration::from_secs(1));

            let source = rodio::source::SineWave::new(340.0)
                .take_duration(Duration::from_millis(500))
                .amplify(0.20);
            stream_handle.play_raw(source.convert_samples()).unwrap();
            std::thread::sleep(Duration::from_secs(1));

            let source = rodio::source::SineWave::new(440.0)
                .take_duration(Duration::from_millis(500))
                .amplify(0.20);
            stream_handle.play_raw(source.convert_samples()).unwrap();
            std::thread::sleep(Duration::from_secs(3));
        }
        AudioCommand::Stop => {
            sink.stop();
        }
    }
}
