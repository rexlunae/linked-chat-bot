//! AI assistant for generating responses using OpenAI.

use anyhow::{Context, Result};
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use super::prompts::get_system_prompt;
use crate::config::UserProfile;
use crate::linkedin::{Conversation, JobOpportunity};

/// AI assistant that generates responses to recruiter messages.
pub struct AIAssistant {
    client: Client<OpenAIConfig>,
    model: String,
}

/// Parsed response from the AI.
#[derive(Debug, Clone)]
pub struct AIResponse {
    /// The message text to send
    pub message: String,
    /// Extracted job opportunity (if any)
    pub job_opportunity: Option<JobOpportunity>,
    /// Interview to schedule (if any)
    pub interview: Option<ParsedInterview>,
}

/// Interview details parsed from AI response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedInterview {
    pub job_title: String,
    pub company: String,
    pub interview_type: String,
    pub date: String,
    pub time: String,
    pub timezone: Option<String>,
    pub duration_minutes: u32,
    pub meeting_link: Option<String>,
    pub location: Option<String>,
    pub interviewers: Vec<String>,
    pub notes: Option<String>,
}

/// Job details parsed from AI response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedJob {
    pub title: String,
    pub company: String,
    pub location: Option<String>,
    pub work_type: Option<String>,
    pub salary_range: Option<String>,
    pub description: Option<String>,
}

impl AIAssistant {
    /// Create a new AI assistant with the given API key.
    pub fn new(api_key: &str, model: &str) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        let client = Client::with_config(config);

        Self {
            client,
            model: model.to_string(),
        }
    }

    /// Generate a response to a recruiter conversation.
    pub async fn generate_response(
        &self,
        conversation: &Conversation,
        user_profile: &UserProfile,
        linkedin_profile_summary: Option<&str>,
    ) -> Result<AIResponse> {
        info!("Generating AI response for conversation");

        // Build user profile context
        let profile_context = self.build_profile_context(user_profile, linkedin_profile_summary);
        let preferences_context = self.build_preferences_context(user_profile);

        // Get system prompt
        let system_prompt = get_system_prompt(&profile_context, &preferences_context);

        // Build conversation context
        let conversation_context = conversation.to_context_string();

        // Create the chat completion request
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(vec![
                ChatCompletionRequestMessage::System(
                    ChatCompletionRequestSystemMessageArgs::default()
                        .content(system_prompt.clone())
                        .build()?,
                ),
                ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessageArgs::default()
                        .content(format!(
                            "Here is the conversation so far:\n\n{}\n\nPlease generate an appropriate response.",
                            conversation_context
                        ))
                        .build()?,
                ),
            ])
            .temperature(0.7_f32)
            .max_tokens(1024_u32)
            .build()
            .context("Failed to build chat completion request")?;

        debug!("Sending request to OpenAI");

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .context("Failed to get response from OpenAI")?;

        let ai_message = response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        debug!("AI response: {}", ai_message);

        // Parse the response for actions
        let ai_response = self.parse_response(&ai_message)?;

        Ok(ai_response)
    }

    /// Build user profile context string.
    fn build_profile_context(
        &self,
        user_profile: &UserProfile,
        linkedin_summary: Option<&str>,
    ) -> String {
        let mut context = String::new();

        context.push_str(&format!("Name: {}\n", user_profile.name));
        context.push_str(&format!("Email: {}\n", user_profile.email));

        if let Some(phone) = &user_profile.phone {
            context.push_str(&format!("Phone: {phone}\n"));
        }

        if let Some(location) = &user_profile.location {
            context.push_str(&format!("Location: {location}\n"));
        }

        if let Some(current_role) = &user_profile.current_role {
            context.push_str(&format!("Current Role: {current_role}\n"));
        }

        if let Some(linkedin) = linkedin_summary {
            context.push_str(&format!("\nLinkedIn Profile:\n{linkedin}\n"));
        }

        context
    }

    /// Build user preferences context string.
    fn build_preferences_context(&self, user_profile: &UserProfile) -> String {
        let mut context = String::new();

        if let Some(target_role) = &user_profile.target_role {
            context.push_str(&format!("Target Role: {target_role}\n"));
        }

        if let Some(availability) = &user_profile.availability {
            context.push_str(&format!("Availability for interviews: {availability}\n"));
        }

        if let Some(salary) = &user_profile.salary_expectations {
            context.push_str(&format!("Salary Expectations: {salary}\n"));
        }

        if let Some(notes) = &user_profile.notes {
            context.push_str(&format!("Additional Notes: {notes}\n"));
        }

        if context.is_empty() {
            context = "No specific preferences provided.".to_string();
        }

        context
    }

    /// Parse the AI response to extract actions and clean message.
    fn parse_response(&self, response: &str) -> Result<AIResponse> {
        let mut message = response.to_string();
        let mut job_opportunity = None;
        let mut interview = None;

        // Extract JSON blocks from the response
        let json_regex = Regex::new(r"```json\s*([\s\S]*?)\s*```")?;

        for cap in json_regex.captures_iter(response) {
            if let Some(json_str) = cap.get(1) {
                let json_text = json_str.as_str();

                // Try to parse as action
                if let Ok(action) = serde_json::from_str::<serde_json::Value>(json_text) {
                    match action.get("action").and_then(|a| a.as_str()) {
                        Some("schedule_interview") => {
                            if let Some(interview_data) = action.get("interview") {
                                if let Ok(parsed) =
                                    serde_json::from_value::<ParsedInterview>(interview_data.clone())
                                {
                                    interview = Some(parsed);
                                }
                            }
                        }
                        Some("extract_job") => {
                            if let Some(job_data) = action.get("job") {
                                if let Ok(parsed) =
                                    serde_json::from_value::<ParsedJob>(job_data.clone())
                                {
                                    let mut job = JobOpportunity::new(parsed.title, parsed.company);
                                    job.location = parsed.location;
                                    job.work_type = parsed.work_type;
                                    job.salary_range = parsed.salary_range;
                                    job.description = parsed.description;
                                    job_opportunity = Some(job);
                                }
                            }
                        }
                        _ => {}
                    }
                }

                // Remove JSON block from the message
                message = message.replace(&cap[0], "");
            }
        }

        // Clean up the message
        message = message.trim().to_string();

        Ok(AIResponse {
            message,
            job_opportunity,
            interview,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_response_simple() {
        let assistant = AIAssistant::new("test-key", "gpt-4");
        let response = "Thank you for reaching out! I'd love to learn more about this opportunity.";

        let parsed = assistant.parse_response(response).unwrap();
        assert_eq!(parsed.message, response);
        assert!(parsed.job_opportunity.is_none());
        assert!(parsed.interview.is_none());
    }

    #[test]
    fn test_parse_response_with_job() {
        let assistant = AIAssistant::new("test-key", "gpt-4");
        let response = r#"Thank you for the details!

```json
{
  "action": "extract_job",
  "job": {
    "title": "Senior Software Engineer",
    "company": "TechCorp",
    "location": "San Francisco",
    "work_type": "hybrid",
    "salary_range": "$150k-$200k"
  }
}
```

I'm very interested in this position."#;

        let parsed = assistant.parse_response(response).unwrap();
        assert!(parsed.message.contains("Thank you"));
        assert!(parsed.message.contains("interested"));
        assert!(parsed.job_opportunity.is_some());

        let job = parsed.job_opportunity.unwrap();
        assert_eq!(job.title, "Senior Software Engineer");
        assert_eq!(job.company, "TechCorp");
    }
}
