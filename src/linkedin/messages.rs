//! LinkedIn message handling.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Direction of a message in a conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageDirection {
    /// Message sent by the user
    Sent,
    /// Message received from another user
    Received,
}

/// A single message in a LinkedIn conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID
    pub id: String,
    /// Message content
    pub content: String,
    /// Sender's name
    pub sender_name: String,
    /// Sender's profile ID
    pub sender_id: String,
    /// Message direction
    pub direction: MessageDirection,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Whether the message has been read
    pub is_read: bool,
}

/// A LinkedIn conversation thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    /// Conversation ID
    pub id: String,
    /// Participant names (excluding the current user)
    pub participants: Vec<String>,
    /// Messages in the conversation
    pub messages: Vec<Message>,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Whether there are unread messages
    pub has_unread: bool,
}

impl Conversation {
    /// Create a new conversation.
    pub fn new(id: String, participants: Vec<String>) -> Self {
        Self {
            id,
            participants,
            messages: Vec::new(),
            last_activity: Utc::now(),
            has_unread: false,
        }
    }

    /// Get the most recent message.
    pub fn last_message(&self) -> Option<&Message> {
        self.messages.last()
    }

    /// Get all unread messages.
    pub fn unread_messages(&self) -> Vec<&Message> {
        self.messages.iter().filter(|m| !m.is_read).collect()
    }

    /// Get the conversation history as a formatted string for AI context.
    pub fn to_context_string(&self) -> String {
        let mut context = format!(
            "Conversation with: {}\n\n",
            self.participants.join(", ")
        );

        for message in &self.messages {
            let direction = match message.direction {
                MessageDirection::Sent => "You",
                MessageDirection::Received => &message.sender_name,
            };
            context.push_str(&format!(
                "[{}] {}: {}\n",
                message.timestamp.format("%Y-%m-%d %H:%M"),
                direction,
                message.content
            ));
        }

        context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_new() {
        let conv = Conversation::new(
            "conv-123".to_string(),
            vec!["John Doe".to_string()],
        );
        assert_eq!(conv.id, "conv-123");
        assert_eq!(conv.participants.len(), 1);
        assert!(conv.messages.is_empty());
        assert!(!conv.has_unread);
    }

    #[test]
    fn test_last_message() {
        let mut conv = Conversation::new(
            "conv-123".to_string(),
            vec!["Jane Recruiter".to_string()],
        );
        assert!(conv.last_message().is_none());

        conv.messages.push(Message {
            id: "msg-1".to_string(),
            content: "Hello!".to_string(),
            sender_name: "Jane Recruiter".to_string(),
            sender_id: "jane-123".to_string(),
            direction: MessageDirection::Received,
            timestamp: Utc::now(),
            is_read: false,
        });

        assert!(conv.last_message().is_some());
        assert_eq!(conv.last_message().unwrap().content, "Hello!");
    }
}
