//! LinkedIn client for browser automation.
//!
//! This module uses the `headless_chrome` crate for browser automation.
//! Note: The browser automation library is synchronous by design, so
//! blocking calls like `std::thread::sleep` are used instead of async
//! alternatives. When used in an async context, consider wrapping calls
//! in `tokio::task::spawn_blocking` for better concurrency.

use anyhow::{Context, Result};
use headless_chrome::{Browser, LaunchOptions, Tab};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

use super::{Conversation, LinkedInProfile, Message};
use crate::config::LinkedInConfig;

/// LinkedIn client that uses browser automation.
///
/// Note: This client uses synchronous browser automation. All methods
/// perform blocking I/O operations.
pub struct LinkedInClient {
    #[allow(dead_code)]
    browser: Browser,
    tab: Arc<Tab>,
    logged_in: bool,
}

impl LinkedInClient {
    /// Create a new LinkedIn client.
    pub fn new() -> Result<Self> {
        info!("Initializing LinkedIn browser client");

        let launch_options = LaunchOptions::default_builder()
            .headless(true)
            .sandbox(false)
            .idle_browser_timeout(Duration::from_secs(300))
            .build()
            .context("Failed to build browser launch options")?;

        let browser = Browser::new(launch_options).context("Failed to launch browser")?;

        let tab = browser
            .new_tab()
            .context("Failed to create new browser tab")?;

        Ok(Self {
            browser,
            tab,
            logged_in: false,
        })
    }

    /// Log in to LinkedIn with the provided credentials.
    pub fn login(&mut self, config: &LinkedInConfig) -> Result<()> {
        info!("Logging in to LinkedIn");

        // Navigate to LinkedIn login page
        self.tab
            .navigate_to("https://www.linkedin.com/login")
            .context("Failed to navigate to LinkedIn login page")?;

        self.tab
            .wait_until_navigated()
            .context("Failed to wait for navigation")?;

        // Wait for the login form to load
        std::thread::sleep(Duration::from_secs(2));

        // Find and fill in the email field
        self.tab
            .wait_for_element("input#username")
            .context("Failed to find username field")?
            .click()
            .context("Failed to click username field")?;

        self.tab
            .type_str(&config.email)
            .context("Failed to type email")?;

        // Find and fill in the password field
        self.tab
            .wait_for_element("input#password")
            .context("Failed to find password field")?
            .click()
            .context("Failed to click password field")?;

        self.tab
            .type_str(&config.password)
            .context("Failed to type password")?;

        // Click the login button
        self.tab
            .wait_for_element("button[type='submit']")
            .context("Failed to find submit button")?
            .click()
            .context("Failed to click submit button")?;

        // Wait for login to complete
        std::thread::sleep(Duration::from_secs(5));

        // Check if login was successful by looking for the feed or home page elements
        let current_url = self.tab.get_url();
        if current_url.contains("feed") || current_url.contains("mynetwork") {
            info!("Successfully logged in to LinkedIn");
            self.logged_in = true;
            Ok(())
        } else if current_url.contains("checkpoint") || current_url.contains("challenge") {
            warn!("LinkedIn is requesting additional verification");
            Err(anyhow::anyhow!(
                "LinkedIn is requesting additional verification (CAPTCHA or 2FA). \
                 Please log in manually first or try again later."
            ))
        } else {
            Err(anyhow::anyhow!(
                "Login may have failed. Current URL: {}",
                current_url
            ))
        }
    }

    /// Check if the client is logged in.
    pub fn is_logged_in(&self) -> bool {
        self.logged_in
    }

    /// Get the current user's LinkedIn profile.
    pub fn get_my_profile(&self) -> Result<LinkedInProfile> {
        if !self.logged_in {
            return Err(anyhow::anyhow!("Not logged in to LinkedIn"));
        }

        info!("Fetching user profile");

        // Navigate to profile page
        self.tab
            .navigate_to("https://www.linkedin.com/in/me/")
            .context("Failed to navigate to profile page")?;

        self.tab
            .wait_until_navigated()
            .context("Failed to wait for navigation")?;

        std::thread::sleep(Duration::from_secs(3));

        // Extract profile information using JavaScript
        let profile_data = self
            .tab
            .evaluate(
                r#"
                (function() {
                    const profile = {
                        name: '',
                        headline: '',
                        location: '',
                        profile_url: window.location.href
                    };

                    // Get name
                    const nameEl = document.querySelector('h1.text-heading-xlarge');
                    if (nameEl) profile.name = nameEl.textContent.trim();

                    // Get headline
                    const headlineEl = document.querySelector('.text-body-medium.break-words');
                    if (headlineEl) profile.headline = headlineEl.textContent.trim();

                    // Get location
                    const locationEl = document.querySelector('.text-body-small.inline.t-black--light.break-words');
                    if (locationEl) profile.location = locationEl.textContent.trim();

                    return JSON.stringify(profile);
                })()
                "#,
                false,
            )
            .context("Failed to extract profile data")?;

        let profile_json = profile_data
            .value
            .as_ref()
            .and_then(|v| v.as_str())
            .unwrap_or("{}");

        debug!("Profile data: {}", profile_json);

        // Parse the extracted data
        let extracted: serde_json::Value =
            serde_json::from_str(profile_json).unwrap_or_default();

        let profile = LinkedInProfile {
            id: String::new(),
            name: extracted["name"].as_str().unwrap_or("").to_string(),
            headline: extracted["headline"].as_str().map(String::from),
            location: extracted["location"].as_str().map(String::from),
            summary: None,
            current_position: None,
            experience: Vec::new(),
            education: Vec::new(),
            skills: Vec::new(),
            profile_url: extracted["profile_url"].as_str().unwrap_or("").to_string(),
            email: None,
            phone: None,
        };

        Ok(profile)
    }

    /// Get recent conversations from LinkedIn messaging.
    pub fn get_conversations(&self) -> Result<Vec<Conversation>> {
        if !self.logged_in {
            return Err(anyhow::anyhow!("Not logged in to LinkedIn"));
        }

        info!("Fetching conversations");

        // Navigate to messaging page
        self.tab
            .navigate_to("https://www.linkedin.com/messaging/")
            .context("Failed to navigate to messaging page")?;

        self.tab
            .wait_until_navigated()
            .context("Failed to wait for navigation")?;

        std::thread::sleep(Duration::from_secs(3));

        // Extract conversation list using JavaScript
        let conversations_data = self
            .tab
            .evaluate(
                r#"
                (function() {
                    const conversations = [];
                    const convElements = document.querySelectorAll('.msg-conversation-listitem');
                    
                    convElements.forEach((el, index) => {
                        if (index >= 20) return; // Limit to 20 conversations
                        
                        const nameEl = el.querySelector('.msg-conversation-listitem__participant-names');
                        const previewEl = el.querySelector('.msg-conversation-card__message-snippet');
                        const timeEl = el.querySelector('.msg-conversation-card__time-stamp');
                        
                        conversations.push({
                            id: 'conv-' + index,
                            participant: nameEl ? nameEl.textContent.trim() : 'Unknown',
                            preview: previewEl ? previewEl.textContent.trim() : '',
                            time: timeEl ? timeEl.textContent.trim() : ''
                        });
                    });
                    
                    return JSON.stringify(conversations);
                })()
                "#,
                false,
            )
            .context("Failed to extract conversations")?;

        let conversations_json = conversations_data
            .value
            .as_ref()
            .and_then(|v| v.as_str())
            .unwrap_or("[]");

        let extracted: Vec<serde_json::Value> =
            serde_json::from_str(conversations_json).unwrap_or_default();

        let conversations = extracted
            .into_iter()
            .map(|v| {
                Conversation::new(
                    v["id"].as_str().unwrap_or("").to_string(),
                    vec![v["participant"].as_str().unwrap_or("").to_string()],
                )
            })
            .collect();

        Ok(conversations)
    }

    /// Get messages from a specific conversation.
    pub fn get_conversation_messages(&self, conversation_id: &str) -> Result<Vec<Message>> {
        if !self.logged_in {
            return Err(anyhow::anyhow!("Not logged in to LinkedIn"));
        }

        info!("Fetching messages for conversation: {}", conversation_id);

        // For now, we'll return an empty vector as message extraction
        // requires more complex DOM manipulation
        // In a production implementation, this would:
        // 1. Click on the conversation in the list
        // 2. Wait for messages to load
        // 3. Extract message content, senders, and timestamps

        Ok(Vec::new())
    }

    /// Send a message in a conversation.
    ///
    /// # Warning
    /// This is a stub implementation that logs the message but does not
    /// actually send it. A full implementation would require:
    /// 1. Navigating to the conversation
    /// 2. Finding the message input field  
    /// 3. Typing the message
    /// 4. Clicking send
    ///
    /// # Arguments
    /// * `conversation_id` - The ID of the conversation
    /// * `message` - The message text to send
    ///
    /// # Returns
    /// Returns Ok(()) but the message is NOT actually sent in this stub.
    pub fn send_message(&self, conversation_id: &str, message: &str) -> Result<()> {
        if !self.logged_in {
            return Err(anyhow::anyhow!("Not logged in to LinkedIn"));
        }

        info!(
            "Sending message to conversation: {} - Message: {}",
            conversation_id, message
        );

        // TODO: Implement actual message sending
        // This requires more complex DOM manipulation:
        // 1. Click on the conversation to open it
        // 2. Wait for the message thread to load
        // 3. Find the message input textarea
        // 4. Focus and type the message
        // 5. Click the send button or press Enter
        // 6. Wait for confirmation that the message was sent
        
        warn!(
            "Message sending is a stub implementation. Message NOT actually sent: {}",
            message
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a browser to be available
    // In CI, they may need to be skipped or run with a mock

    #[test]
    #[ignore = "Requires browser"]
    fn test_client_creation() {
        let client = LinkedInClient::new();
        assert!(client.is_ok());
    }
}
