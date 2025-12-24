//! LoomWM AI Integration
//!
//! Handles intent interpretation and generative UI:
//! - Parse natural language intents
//! - Suggest node arrangements
//! - Generate UI elements based on context
//! - Infer semantic connections between nodes

pub mod generator;
pub mod intent;
pub mod suggestions;

pub use generator::UiGenerator;
pub use intent::{Intent, IntentParser};
pub use suggestions::SuggestionEngine;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AiError {
    #[error("Failed to parse intent: {0}")]
    ParseError(String),

    #[error("AI service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Generation failed: {0}")]
    GenerationFailed(String),
}

pub type Result<T> = std::result::Result<T, AiError>;
