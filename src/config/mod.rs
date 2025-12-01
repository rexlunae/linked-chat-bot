//! Configuration module for the LinkedIn chat bot.
//!
//! Handles loading and managing configuration from environment variables
//! and configuration files.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// LinkedIn credentials
    pub linkedin: LinkedInConfig,
    /// OpenAI API configuration
    pub openai: OpenAIConfig,
    /// Google Calendar configuration
    pub google_calendar: GoogleCalendarConfig,
    /// User profile information to share with recruiters
    pub user_profile: UserProfile,
}

/// LinkedIn login credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedInConfig {
    /// LinkedIn email or username
    pub email: String,
    /// LinkedIn password
    pub password: String,
}

/// OpenAI API configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    /// OpenAI API key
    pub api_key: String,
    /// Model to use (default: gpt-4)
    #[serde(default = "default_model")]
    pub model: String,
}

fn default_model() -> String {
    "gpt-4".to_string()
}

/// Google Calendar configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleCalendarConfig {
    /// Path to Google OAuth2 credentials JSON file
    pub credentials_path: PathBuf,
    /// Calendar ID to add events to (default: primary)
    #[serde(default = "default_calendar_id")]
    pub calendar_id: String,
}

fn default_calendar_id() -> String {
    "primary".to_string()
}

/// User's profile information that can be shared with recruiters.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserProfile {
    /// User's full name
    pub name: String,
    /// User's email for contact
    pub email: String,
    /// User's phone number
    pub phone: Option<String>,
    /// User's location/timezone
    pub location: Option<String>,
    /// User's preferred availability for interviews
    pub availability: Option<String>,
    /// User's current role/title
    pub current_role: Option<String>,
    /// User's target role/title
    pub target_role: Option<String>,
    /// User's salary expectations
    pub salary_expectations: Option<String>,
    /// Additional notes or preferences
    pub notes: Option<String>,
}

impl Config {
    /// Load configuration from environment variables.
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        let linkedin = LinkedInConfig {
            email: std::env::var("LINKEDIN_EMAIL")
                .context("LINKEDIN_EMAIL environment variable not set")?,
            password: std::env::var("LINKEDIN_PASSWORD")
                .context("LINKEDIN_PASSWORD environment variable not set")?,
        };

        let openai = OpenAIConfig {
            api_key: std::env::var("OPENAI_API_KEY")
                .context("OPENAI_API_KEY environment variable not set")?,
            model: std::env::var("OPENAI_MODEL").unwrap_or_else(|_| default_model()),
        };

        let google_calendar = GoogleCalendarConfig {
            credentials_path: PathBuf::from(
                std::env::var("GOOGLE_CREDENTIALS_PATH")
                    .unwrap_or_else(|_| "google_credentials.json".to_string()),
            ),
            calendar_id: std::env::var("GOOGLE_CALENDAR_ID")
                .unwrap_or_else(|_| default_calendar_id()),
        };

        let user_profile = UserProfile {
            name: std::env::var("USER_NAME").unwrap_or_default(),
            email: std::env::var("USER_EMAIL").unwrap_or_default(),
            phone: std::env::var("USER_PHONE").ok(),
            location: std::env::var("USER_LOCATION").ok(),
            availability: std::env::var("USER_AVAILABILITY").ok(),
            current_role: std::env::var("USER_CURRENT_ROLE").ok(),
            target_role: std::env::var("USER_TARGET_ROLE").ok(),
            salary_expectations: std::env::var("USER_SALARY_EXPECTATIONS").ok(),
            notes: std::env::var("USER_NOTES").ok(),
        };

        Ok(Config {
            linkedin,
            openai,
            google_calendar,
            user_profile,
        })
    }

    /// Load configuration from a JSON file.
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let config: Config = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_model() {
        assert_eq!(default_model(), "gpt-4");
    }

    #[test]
    fn test_default_calendar_id() {
        assert_eq!(default_calendar_id(), "primary");
    }

    #[test]
    fn test_user_profile_default() {
        let profile = UserProfile::default();
        assert!(profile.name.is_empty());
        assert!(profile.email.is_empty());
        assert!(profile.phone.is_none());
    }
}
