mod app;
mod slack;
mod ui;

use anyhow::Result;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    let log = std::fs::File::create("/tmp/slate.log")?;
    tracing_subscriber::fmt()
        .with_writer(std::sync::Mutex::new(log))
        .init();

    let token = std::env::var("SLACK_TOKEN")
        .map_err(|_| anyhow::anyhow!("set SLACK_TOKEN (xoxp-/xoxb- token)"))?;
    let client = slack::Client::new(token);

    let (tx, rx) = mpsc::unbounded_channel::<app::Event>();

    {
        let (client, tx) = (client.clone(), tx.clone());
        tokio::spawn(async move {
            match client.conversations_list().await {
                Ok(chans) => {
                    let _ = tx.send(app::Event::Channels(chans));
                }
                Err(e) => {
                    let _ = tx.send(app::Event::Error(e.to_string()));
                }
            }
        });
    }

    ui::run(client, tx, rx).await
}
