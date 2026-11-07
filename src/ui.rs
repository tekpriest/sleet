use anyhow::Result;
use crossterm::event::{Event as CEvent, EventStream, KeyCode, KeyModifiers};
use futures_util::StreamExt;
use ratatui::DefaultTerminal;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::app::{App, Event};
use crate::slack::Client;

pub async fn run(
    client: Client,
    tx: UnboundedSender<Event>,
    mut rx: UnboundedReceiver<Event>,
) -> Result<()> {
    let mut term = ratatui::init();
    let res = event_loop(&mut term, client, tx, &mut rx).await;
    ratatui::restore();
    res
}

async fn event_loop(
    term: &mut DefaultTerminal,
    client: Client,
    tx: UnboundedSender<Event>,
    rx: &mut UnboundedReceiver<Event>,
) -> Result<()> {
    let mut app = App::default();
    let mut keys = EventStream::new();

    loop {
        term.draw(|f| draw(f, &app))?;

        tokio::select! {
                    Some(ev) = rx.recv() => app.apply(ev),
                    Some(Ok(CEvent::Key(k))) = keys.next() => {
                        match (k.code, k.modifiers) {
                            (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(()),
                            (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::CONTROL) => {app.next(); load(&client, &tx, &app);}
                            (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::CONTROL) => {app.prev(); load(&client, &tx, &app);}
                            (KeyCode::Enter, _) => {
                                if let Some(ch) = app.current().cloned() {
                                        let text = std::mem::take(&mut app.input);
        if !text.is_empty() {
                                        let (c, tx) = (client.clone(), tx.clone());
                                        tokio::spawn(async move {
        if let Err(e) = c.post_message(&ch.id, &text).await {
                                                let _ = tx.send(Event::Error(e.to_string()));
        }
                                        });
                                    }
                                }
                            }
                    (KeyCode::Backspace, _) => {app.input.pop();}
                    (KeyCode::Char(c), _) => app.input.push(c),
                            _ => {}
                        }
                    }
                }
    }
}

fn draw(_f: &mut ratatui::Frame, _app: &App) {}

fn load(_client: &Client, _tx: &UnboundedSender<Event>, _app: &App) {}
