//! AI module for generating responses to recruiter messages.

mod assistant;
mod prompts;

pub use assistant::{AIAssistant, ParsedInterview, ParsedJob};
