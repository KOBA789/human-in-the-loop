use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;
use tokio::time::{sleep, timeout};

use crate::tools::Human;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SlackMessage {
    pub text: String,
    pub user: String,
    pub ts: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SlackResponse {
    pub ok: bool,
    pub channel: Option<String>,
    pub ts: Option<String>,
    pub messages: Option<Vec<SlackMessage>>,
    pub error: Option<String>,
}

pub struct HumanInSlack {
    token: String,
    channel_id: String,
    user_id: String,
    client: Arc<Client>,
    thread_ts: OnceCell<String>,
}

impl HumanInSlack {
    pub fn new(token: String, channel_id: String, user_id: String) -> Self {
        Self {
            token,
            channel_id,
            user_id,
            client: Arc::new(Client::new()),
            thread_ts: OnceCell::new(),
        }
    }

    async fn send_message(&self, text: &str, thread_ts: Option<&str>) -> anyhow::Result<String> {
        let mut form = vec![
            ("token", self.token.as_str()),
            ("channel", &self.channel_id),
            ("text", text),
        ];

        if let Some(ts) = thread_ts {
            form.push(("thread_ts", ts));
        }

        let response = self
            .client
            .post("https://slack.com/api/chat.postMessage")
            .form(&form)
            .send()
            .await?;

        let slack_response: SlackResponse = response.json().await?;
        
        if !slack_response.ok {
            return Err(anyhow::anyhow!(
                "Failed to send Slack message: {}",
                slack_response.error.unwrap_or_else(|| "Unknown error".to_string())
            ));
        }

        Ok(slack_response.ts.unwrap_or_default())
    }

    async fn get_messages_after(&self, after_ts: &str, thread_ts: Option<&str>) -> anyhow::Result<Vec<SlackMessage>> {
        let mut form = vec![
            ("token", self.token.as_str()),
            ("channel", &self.channel_id),
            ("oldest", after_ts),
            ("limit", "100"),
        ];

        if let Some(ts) = thread_ts {
            form.push(("ts", ts));
        }

        let endpoint = if thread_ts.is_some() {
            "https://slack.com/api/conversations.replies"
        } else {
            "https://slack.com/api/conversations.history"
        };

        let response = self
            .client
            .post(endpoint)
            .form(&form)
            .send()
            .await?;

        let slack_response: SlackResponse = response.json().await?;
        
        if !slack_response.ok {
            return Err(anyhow::anyhow!(
                "Failed to get Slack messages: {}",
                slack_response.error.unwrap_or_else(|| "Unknown error".to_string())
            ));
        }

        Ok(slack_response.messages.unwrap_or_default())
    }

    async fn wait_for_reply(&self, after_ts: &str, thread_ts: Option<&str>) -> anyhow::Result<String> {
        const MAX_WAIT_TIME: Duration = Duration::from_secs(300); // 5 minutes
        const INITIAL_POLL_INTERVAL: Duration = Duration::from_secs(10);
        const MAX_POLL_INTERVAL: Duration = Duration::from_secs(30);

        let result = timeout(MAX_WAIT_TIME, async {
            let mut poll_interval = INITIAL_POLL_INTERVAL;
            let mut retry_count = 0;
            
            loop {
                match self.get_messages_after(after_ts, thread_ts).await {
                    Ok(messages) => {
                        // Reset poll interval on success
                        poll_interval = INITIAL_POLL_INTERVAL;
                        retry_count = 0;
                        
                        // Look for messages from the specified user that are newer than our message
                        for message in messages {
                            if message.user == self.user_id && message.ts.as_str() > after_ts {
                                return Ok(message.text);
                            }
                        }
                    }
                    Err(e) => {
                        // Exponential backoff on error (likely rate limit)
                        if e.to_string().contains("ratelimited") {
                            retry_count += 1;
                            poll_interval = std::cmp::min(
                                Duration::from_secs(10 * 2_u64.pow(retry_count.min(3))),
                                MAX_POLL_INTERVAL
                            );
                            eprintln!("Rate limited, backing off to {:?}", poll_interval);
                        } else {
                            return Err(e);
                        }
                    }
                }
                
                sleep(poll_interval).await;
            }
        }).await;

        match result {
            Ok(reply) => reply,
            Err(_) => Err(anyhow::anyhow!("Timeout waiting for human reply in Slack")),
        }
    }
}

#[async_trait::async_trait]
impl Human for HumanInSlack {
    async fn ask(&self, question: &str) -> anyhow::Result<String> {
        let message_text = format!("<@{}> {}", self.user_id, question);
        
        // Check if this is the first question (no thread exists yet)
        let sent_ts = if let Some(thread_ts) = self.thread_ts.get() {
            // Thread already exists, send message to thread
            self.send_message(&message_text, Some(thread_ts)).await?
        } else {
            // First question, create new thread and initialize thread_ts
            let ts = self.send_message(&message_text, None).await?;
            self.thread_ts.set(ts.clone()).map_err(|_| {
                anyhow::anyhow!("Failed to set thread timestamp")
            })?;
            ts
        };
        
        // Wait for the human's reply
        let thread_ts = self.thread_ts.get().map(|s| s.as_str());
        self.wait_for_reply(&sent_ts, thread_ts).await
    }
}