use iced::{
    time,
    widget::{button, row, text, Column},
    Element, Subscription,
};
use std::time::{Duration, Instant};

struct PomodoroTimer {
    time_left: u32, // Time left in seconds
    end_time: Option<Instant>,
    work_periods: u32,
    is_running: bool,
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
            time_left: 25 * 60,
            end_time: None,
            work_periods: 0,
            is_running: false,
            is_work_period: true,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let buttons = row![]
            .push(
                button(if self.is_running { "Pause" } else { "Start" })
                    .on_press(Message::StartStop),
            )
            .push(button("Reset").on_press(Message::Reset));
        Column::new()
            .push(
                text(format!(
                    "{:02}:{:02}",
                    self.time_left / 60,
                    self.time_left % 60
                ))
                .size(50),
            )
            .push(buttons)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let tick = match self.is_running {
            true => time::every(Duration::from_millis(10)).map(Message::Tick),
            false => Subscription::none(),
        };
        tick
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Tick(now) => {
                if self.is_running && self.time_left > 0 {
                    self.time_left = (self.end_time.unwrap() - now).as_secs() as u32;
                }
                if self.time_left == 0 {
                    self.work_periods += 1;
                    self.is_work_period = self.work_periods % 4 == 3;
                    self.time_left = if self.is_work_period { 25 * 60 } else { 5 * 60 };
                    self.is_running = false;
                }
            }
            Message::StartStop => {
                self.is_running = !self.is_running;
                if self.is_running {
                    self.end_time =
                        Some(Instant::now() + Duration::from_secs(self.time_left as u64));
                }
            }
            Message::Reset => {
                self.is_running = false;
                self.is_work_period = true;
                self.time_left = 25 * 60;
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
    iced::application("Pomodoro Timer", PomodoroTimer::update, PomodoroTimer::view)
        .subscription(PomodoroTimer::subscription)
        .theme(PomodoroTimer::theme)
        .run()
}
