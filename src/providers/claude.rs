use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{AgentContext, AiProvider, SkillContext};

pub struct ClaudeProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
}

#[derive(Deserialize)]
struct ChatResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
}

impl ClaudeProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
        }
    }

    pub fn from_env() -> Option<Self> {
        std::env::var("ANTHROPIC_API_KEY")
            .or_else(|_| std::env::var("CLAUDE_API_KEY"))
            .ok()
            .map(Self::new)
    }

    async fn chat(&self, prompt: &str, model: &str) -> Result<String> {
        let request = ChatRequest {
            model: model.to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: 4096,
        };

        let response = self
            .client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Claude API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Claude API error: {} - {}", status, body);
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse Claude API response")?;

        let text = chat_response
            .content
            .iter()
            .filter(|c| c.block_type == "text")
            .map(|c| c.text.clone().unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n");

        if text.is_empty() {
            anyhow::bail!("No text content in Claude API response");
        }

        Ok(text)
    }
}

#[async_trait]
impl AiProvider for ClaudeProvider {
    fn name(&self) -> &str {
        "claude"
    }

    async fn generate_skill(&self, context: &SkillContext) -> Result<String> {
        let model = std::env::var("CLAUDE_MODEL").unwrap_or_else(|_| "claude-3-5-haiku-20241107".to_string());

        let prompt = format!(
            r#"Generate a Claude Code SKILL.md file for a {} project.

Skill name: {}
Description: {}

Commands to include:
{}

Generate a SKILL.md with frontmatter (name, description, allowed-tools) and relevant command sections.
Keep it concise and practical.
"#,
            context.project_type,
            context.skill_name,
            context.description,
            context.commands.join("\n")
        );

        self.chat(&prompt, &model).await
    }

    async fn generate_agent(&self, context: &AgentContext) -> Result<String> {
        let model = std::env::var("CLAUDE_MODEL").unwrap_or_else(|_| "claude-3-5-haiku-20241107".to_string());

        let prompt = format!(
            r#"Generate a Claude Code subagent configuration for:

Agent name: {}
Description: {}

Tasks:
{}

Provide a JSON agent configuration with name, description, instructions, and tools.
"#,
            context.agent_name,
            context.description,
            context.tasks.join("\n")
        );

        self.chat(&prompt, &model).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_provider_from_env_missing() {
        unsafe {
            std::env::remove_var("ANTHROPIC_API_KEY");
            std::env::remove_var("CLAUDE_API_KEY");
        }

        let provider = ClaudeProvider::from_env();
        assert!(provider.is_none());
    }
}
