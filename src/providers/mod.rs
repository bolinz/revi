use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

pub mod minimax;
pub mod ollama;
pub mod claude;

pub use minimax::MiniMaxProvider;
pub use ollama::OllamaProvider;
pub use claude::ClaudeProvider;

#[async_trait]
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn generate_skill(&self, context: &SkillContext) -> Result<String>;
    async fn generate_agent(&self, context: &AgentContext) -> Result<String>;
}

#[derive(Clone, Debug, Default)]
pub struct SkillContext {
    pub skill_name: String,
    pub description: String,
    pub project_type: String,
    pub commands: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct AgentContext {
    pub agent_name: String,
    pub description: String,
    pub tasks: Vec<String>,
}

pub struct ProviderRegistry {
    providers: HashMap<String, Box<dyn AiProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register<P: AiProvider + 'static>(&mut self, provider: P) {
        let name = provider.name().to_string();
        self.providers.insert(name, Box::new(provider));
    }

    pub fn get(&self, name: &str) -> Option<&dyn AiProvider> {
        self.providers.get(name).map(|p| p.as_ref())
    }

    pub fn names(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a provider by name.
///
/// For "minimax", MINIMAX_API_KEY env var is required.
/// For "ollama", uses OLLAMA_BASE_URL and OLLAMA_MODEL env vars (defaults to localhost:11434 and llama3).
/// For "claude", ANTHROPIC_API_KEY or CLAUDE_API_KEY env var is required.
pub fn create_provider(name: &str) -> Result<Box<dyn AiProvider>> {
    match name {
        "minimax" => {
            let api_key = std::env::var("MINIMAX_API_KEY")
                .map_err(|_| anyhow::anyhow!("MINIMAX_API_KEY is required"))?;
            Ok(Box::new(minimax::MiniMaxProvider::new(api_key)))
        }
        "ollama" => {
            let base_url = std::env::var("OLLAMA_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string());
            let model = std::env::var("OLLAMA_MODEL")
                .unwrap_or_else(|_| "llama3".to_string());
            Ok(Box::new(ollama::OllamaProvider::new(base_url, model)))
        }
        "claude" => {
            let api_key = std::env::var("ANTHROPIC_API_KEY")
                .or_else(|_| std::env::var("CLAUDE_API_KEY"))
                .map_err(|_| anyhow::anyhow!("ANTHROPIC_API_KEY or CLAUDE_API_KEY is required"))?;
            Ok(Box::new(claude::ClaudeProvider::new(api_key)))
        }
        _ => anyhow::bail!("Unknown provider: {}. Available: minimax, ollama, claude", name),
    }
}
