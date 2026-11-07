use anyhow::{Result, bail};
use serde::Deserialize;

const API: &str = "https://slack.com/api";

#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    token: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Channel {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub is_im: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Message {
    pub ts: String,
    #[serde(default)]
    pub user: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub thread_ts: Option<String>,
}

#[derive(Deserialize)]
struct Envelope<T> {
    ok: bool,
    error: Option<String>,
    #[serde(flatten)]
    data: Option<T>,
}

#[derive(Deserialize)]
struct ChannelsResp {
    channels: Vec<Channel>,
}

#[derive(Deserialize)]
struct HistoryResp {
    messages: Vec<Message>,
}

impl Client {
    pub fn new(token: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            token,
        }
    }

    async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        q: &[(&str, &str)],
    ) -> Result<T> {
        let env: Envelope<T> = self
            .http
            .get(format!("{API}/{method}", API = API, method = method))
            .bearer_auth(&self.token)
            .query(q)
            .send()
            .await?
            .json()
            .await?;

        match (env.ok, env.data) {
            (true, Some(d)) => Ok(d),
            _ => bail!("slack {method}: {}", env.error.unwrap_or_default()),
        }
    }

    pub async fn conversations_list(&self) -> Result<Vec<Channel>> {
        let r: ChannelsResp = self
            .get(
                "conversations.list",
                &[
                    ("types", "public_channel.private_channel.im"),
                    ("limit", "200"),
                ],
            )
            .await?;

        Ok(r.channels)
    }

    pub async fn history(&self, channel: &str) -> Result<Vec<Message>> {
        let r: HistoryResp = self
            .get(
                "conversations.history",
                &[("channel", channel), ("limit", "50")],
            )
            .await?;
        let mut m = r.messages;
        m.reverse();
        Ok(m)
    }

    pub async fn post_message(&self, channel: &str, text: &str) -> Result<()> {
        let res: serde_json::Value = self
            .http
            .post(format!("{}/chat.postMessage", API))
            .bearer_auth(&self.token)
            .json(&serde_json::json!({"channel": channel, "text": text}))
            .send()
            .await?
            .json()
            .await?;
        if res["ok"].as_bool() != Some(true) {
            bail!("chat.postMessage: {}", res["error"]);
        }
        Ok(())
    }
}
