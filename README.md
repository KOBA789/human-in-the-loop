# Human-in-the-Loop MCP Server

An MCP (Model Context Protocol) server that allows AI assistants to ask questions to humans via Discord or Slack.

<img width="845" alt="Screenshot 2025-06-23 at 18 25 43" src="https://github.com/user-attachments/assets/dcdbb1a7-cb71-446e-b44d-bfe637059acb" />

## Overview

This MCP server is used when AI assistants need human input or judgment during their work. For example:

- When having an LLM create documentation, the AI designs the structure while humans provide specific content
- When the AI needs confirmation on uncertain decisions
- When specialized knowledge or personal information is required

## Requirements

- Rust (1.70 or higher)
- Discord (account and bot) or Slack (account and app)
- MCP-compatible AI client (Claude Desktop, Copilot Edits, etc.)

## Setup

### 1. Bot setup

Choose either Discord or Slack:

#### Discord Bot

1. Go to [Discord Developer Portal](https://discord.com/developers/applications)
2. Create a new application
3. Create a bot in the Bot section and obtain the token
4. Set required permissions:
   - Send Messages
   - Create Public Threads
   - Read Message History
5. Enable privileged gateway intents in Bot section:
   - Message Content Intent

#### Slack App

1. Go to [Slack API](https://api.slack.com/apps) and create a new app
2. Choose "From scratch" and select your workspace
3. Go to "OAuth & Permissions" and add these Bot Token Scopes:
   - `chat:write` - Send messages
   - `channels:history` - Read public channel history
   - `channels:read` - View basic information about public channels
   - `groups:history` - Read private channel history (if using private channels)
   - `groups:read` - View basic information about private channels (if using private channels)
   - `users:read` - View people in workspace
4. Install the app to your workspace
5. Copy the "Bot User OAuth Token" (starts with `xoxb-`)

### 2. Install

```bash
cargo install --git https://github.com/KOBA789/human-in-the-loop.git
```

## Connecting with MCP Clients

### Claude Desktop Configuration

#### Discord
Add the following to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "human-in-the-loop-discord": {
      "command": "human-in-the-loop",
      "args": [
        "discord",
        "--channel-id", "your-discord-channel-id",
        "--user-id", "your-discord-user-id"
      ],
      "env": {
        "DISCORD_TOKEN": "your-discord-bot-token"
      }
    }
  }
}
```

#### Slack
Add the following to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "human-in-the-loop-slack": {
      "command": "human-in-the-loop",
      "args": [
        "slack",
        "--channel-id", "your-slack-channel-id",
        "--user-id", "your-slack-user-id"
      ],
      "env": {
        "SLACK_TOKEN": "your-slack-bot-token"
      }
    }
  }
}
```

### Claude Code Configuration

#### Discord
Add to your MCP settings:

```json
{
  "mcpServers": {
    "human-in-the-loop-discord": {
      "command": "human-in-the-loop",
      "args": [
        "discord",
        "--channel-id", "your-discord-channel-id",
        "--user-id", "your-discord-user-id"
      ]
    }
  }
}
```

Set the Discord token as an environment variable before running Claude Code:

```bash
export DISCORD_TOKEN="your-discord-bot-token"
claude
```

#### Slack
Add to your MCP settings:

```json
{
  "mcpServers": {
    "human-in-the-loop-slack": {
      "command": "human-in-the-loop",
      "args": [
        "slack",
        "--channel-id", "your-slack-channel-id",
        "--user-id", "your-slack-user-id"
      ]
    }
  }
}
```

Set the Slack token as an environment variable before running Claude Code:

```bash
export SLACK_TOKEN="your-slack-bot-token"
claude
```

Note: The server automatically reads tokens from environment variables (`DISCORD_TOKEN` or `SLACK_TOKEN`). You can also pass them via `--token` argument if needed.

### Usage

AI assistants can ask questions to humans using the `ask_human` tool:

```
Human: Please create a documentation outline. You can ask the human as you need.
Assistant: I'll create a documentation outline. Let me ask you some questions first.
[Uses ask_human tool]
```

The AI posts questions in Discord or Slack and mentions the specified user. When the user replies, the response is returned to the AI.

## How It Works

1. AI assistant calls the `ask_human` tool
2. MCP server creates a thread in the specified channel (or uses existing thread)
3. Posts the question and mentions the specified user
4. Waits for user's reply
5. Returns the reply content to the AI assistant

## Finding IDs

### Discord IDs

#### Getting Channel ID
1. Enable Developer Mode in Discord (Settings → Advanced → Developer Mode)
2. Right-click on channel → "Copy ID"

#### Getting User ID
1. Right-click on user → "Copy ID"

### Slack IDs

#### Getting Channel ID
1. Right-click on channel → "Copy link"
2. Extract the channel ID from the URL (e.g., `C1234567890`)

#### Getting User ID  
1. Click on user profile → "More" → "Copy member ID"
2. Or extract from URL when viewing user profile (e.g., `U1234567890`)

## Roadmap

- **Future Migration to MCP Elicitation**: Once MCP's Elicitation implementation becomes more widespread and standardized, we plan to migrate the UI from Discord/Slack to native MCP Elicitation. This will provide a more integrated experience directly within MCP-compatible clients.
