//! LinkedIn client module for browser automation.
//!
//! This module handles LinkedIn login, profile fetching, and message handling
//! using headless browser automation.

mod client;
mod messages;
mod profile;
mod types;

pub use client::LinkedInClient;
pub use messages::{Conversation, Message, MessageDirection};
pub use profile::LinkedInProfile;
pub use types::*;
