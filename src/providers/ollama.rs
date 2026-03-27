use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{AgentContext, AiProvider, SkillContext};

pub struct OllamaProvider {
    client: Client,
    base_url: String,
    model: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

impl OllamaProvider {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            model,
        }
    }

    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var("OLLAMA_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());
        let model = std::env::var("OLLAMA_MODEL")
            .unwrap_or_else(|_| "llama3".to_string());
        Some(Self::new(base_url, model))
    }

    async fn chat(&self, prompt: &str) -> Result<String> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            stream: false,
        };

        let url = format!("{}/api/chat", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Ollama API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Ollama API error: {} - {}", status, body);
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse Ollama API response")?;

        Ok(chat_response.message.content)
    }
}

#[async_trait]
impl AiProvider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    async fn generate_skill(&self, context: &SkillContext) -> Result<String> {
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

        self.chat(&prompt).await
    }

    async fn generate_agent(&self, context: &AgentContext) -> Result<String> {
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

        self.chat(&prompt).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_provider_from_env_default() {
        // Clear any existing env vars (unsafe but needed for test isolation)
        unsafe {
            std::env::remove_var("OLLAMA_BASE_URL");
            std::env::remove_var("OLLAMA_MODEL");
        }

        let provider = OllamaProvider::from_env();
        assert!(provider.is_some());

        let provider = provider.unwrap();
        assert_eq!(provider.base_url, "http://localhost:11434");
        assert_eq!(provider.model, "llama3");
    }
}
