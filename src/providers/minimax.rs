use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{AgentContext, AiProvider, SkillContext};

pub struct MiniMaxProvider {
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
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

impl MiniMaxProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.minimax.chat/v1/chat/completions".to_string(),
        }
    }

    async fn chat(&self, prompt: &str) -> Result<String> {
        let request = ChatRequest {
            model: "mini-max-01".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        };

        let response = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to MiniMax API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("MiniMax API error: {} - {}", status, body);
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse MiniMax API response")?;

        chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| anyhow::anyhow!("No response content from MiniMax API"))
    }
}

#[async_trait]
impl AiProvider for MiniMaxProvider {
    fn name(&self) -> &str {
        "minimax"
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

pub fn client_from_env() -> Option<MiniMaxProvider> {
    std::env::var("MINIMAX_API_KEY")
        .ok()
        .filter(|key| !key.is_empty())
        .map(MiniMaxProvider::new)
}

impl SkillContext {
    pub fn new(skill_name: &str, description: &str, project_type: &str) -> Self {
        Self {
            skill_name: skill_name.to_string(),
            description: description.to_string(),
            project_type: project_type.to_string(),
            commands: Vec::new(),
        }
    }

    pub fn add_command(&mut self, command: &str) {
        self.commands.push(command.to_string());
    }
}

impl AgentContext {
    pub fn new(agent_name: &str, description: &str) -> Self {
        Self {
            agent_name: agent_name.to_string(),
            description: description.to_string(),
            tasks: Vec::new(),
        }
    }

    pub fn add_task(&mut self, task: &str) {
        self.tasks.push(task.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_context_new() {
        let mut ctx = SkillContext::new("test-skill", "A test skill", "Rust");
        ctx.add_command("cargo build");
        ctx.add_command("cargo test");

        assert_eq!(ctx.skill_name, "test-skill");
        assert_eq!(ctx.commands.len(), 2);
    }

    #[test]
    fn test_agent_context_new() {
        let mut ctx = AgentContext::new("test-agent", "A test agent");
        ctx.add_task("Run tests");
        ctx.add_task("Build project");

        assert_eq!(ctx.agent_name, "test-agent");
        assert_eq!(ctx.tasks.len(), 2);
    }

    #[test]
    fn test_client_from_env_missing() {
        // Safety: This test only modifies an environment variable that we own
        unsafe {
            std::env::remove_var("MINIMAX_API_KEY");
        }
        assert!(client_from_env().is_none());
    }
}
