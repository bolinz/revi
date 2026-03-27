use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

pub mod minimax;

pub use minimax::MiniMaxProvider;

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

pub fn create_provider(name: &str, api_key: &str) -> Result<Box<dyn AiProvider>> {
    match name {
        "minimax" => Ok(Box::new(minimax::MiniMaxProvider::new(api_key.to_string()))),
        _ => anyhow::bail!("Unknown provider: {}", name),
    }
}
