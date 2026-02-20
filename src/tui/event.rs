use crossterm::event::{self, Event};
use std::time::Duration;

pub fn poll_event() -> anyhow::Result<Option<Event>> {
    if event::poll(Duration::from_millis(250))? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}
