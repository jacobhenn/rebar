use chrono::{DateTime, Local};
use std::fmt;

pub struct Time(pub DateTime<Local>);

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(time) = self;
        let formatted_time = time.format("%A (%B) %F %T (UTC%z)");
        write!(f, "^bg(#a3be8c) {} ^bg()", formatted_time)
    }
}

impl Time {
    pub fn new() -> Self {
        Self(Local::now())
    }
}
