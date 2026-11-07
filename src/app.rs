use crate::slack::{Channel, Message};

pub enum Event {
    Channels(Vec<Channel>),
    History(String, Vec<Message>),
    Error(String),
}

#[derive(Default)]
pub struct App {
    pub channels: Vec<Channel>,
    pub selected: usize,
    pub messages: Vec<Message>,
    pub input: String,
    pub status: String,
}

impl App {
    pub fn apply(&mut self, ev: Event) {
        match ev {
            Event::Channels(c) => self.channels = c,
            Event::History(_, m) => self.messages = m,
            Event::Error(e) => self.status = e,
        }
    }

    pub fn current(&self) -> Option<&Channel> {
        self.channels.get(self.selected)
    }

    pub fn next(&mut self) {
        if !self.channels.is_empty() {
            self.selected = (self.selected + 1) % self.channels.len();
        }
    }

    pub fn prev(&mut self) {
        if !self.channels.is_empty() {
            self.selected = (self.selected + self.channels.len() - 1) % self.channels.len();
        }
    }
}
