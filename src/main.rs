mod discord;
mod slack;
mod tools;

use clap::{Parser, Subcommand};
use discord::HumanInDiscord;
use slack::HumanInSlack;
use rmcp::serve_server;
use serenity::all::{ChannelId, UserId};
use tokio::io::{stdin, stdout};

#[derive(Debug, Parser)]
struct Args {
    #[clap(subcommand)]
    platform: Platform,
}

#[derive(Debug, Subcommand)]
enum Platform {
    Discord {
        #[clap(long, env = "DISCORD_TOKEN")]
        token: String,
        #[clap(long, env = "DISCORD_CHANNEL_ID")]
        channel_id: ChannelId,
        #[clap(long, env = "DISCORD_USER_ID")]
        user_id: UserId,
    },
    Slack {
        #[clap(long, env = "SLACK_TOKEN")]
        token: String,
        #[clap(long, env = "SLACK_CHANNEL_ID")]
        channel_id: String,
        #[clap(long, env = "SLACK_USER_ID")]
        user_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Args { platform } = Args::parse();

    match platform {
        Platform::Discord { token, channel_id, user_id } => {
            let human = HumanInDiscord::new(user_id, channel_id);
            let discord = discord::start(&token, human.handler().clone());

            let handler = tools::HumanInTheLoop::new(human);
            let transport = (stdin(), stdout());
            let mcp = serve_server(handler, transport).await?;

            tokio::select! {
                res = mcp.waiting() => {
                    res?;
                },
                res = discord => {
                    res?;
                },
            }
        }
        Platform::Slack { token, channel_id, user_id } => {
            let human = HumanInSlack::new(token, channel_id, user_id);

            let handler = tools::HumanInTheLoop::new(human);
            let transport = (stdin(), stdout());
            let mcp = serve_server(handler, transport).await?;

            // For Slack, we only need to wait for the MCP server since Slack uses webhooks/polling
            mcp.waiting().await?;
        }
    }
    Ok(())
}
