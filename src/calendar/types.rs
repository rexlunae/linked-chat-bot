//! Google Calendar event types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a calendar event for an interview.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterviewEvent {
    /// Event title
    pub title: String,
    /// Event description
    pub description: String,
    /// Start time
    pub start_time: DateTime<Utc>,
    /// End time
    pub end_time: DateTime<Utc>,
    /// Location (physical address or video link)
    pub location: Option<String>,
    /// Meeting link (for video interviews)
    pub meeting_link: Option<String>,
    /// Attendees (interviewers)
    pub attendees: Vec<String>,
}

impl InterviewEvent {
    /// Create a new interview event.
    pub fn new(
        title: String,
        description: String,
        start_time: DateTime<Utc>,
        duration_minutes: u32,
    ) -> Self {
        let end_time = start_time + chrono::Duration::minutes(i64::from(duration_minutes));

        Self {
            title,
            description,
            start_time,
            end_time,
            location: None,
            meeting_link: None,
            attendees: Vec::new(),
        }
    }

    /// Set the location.
    pub fn with_location(mut self, location: String) -> Self {
        self.location = Some(location);
        self
    }

    /// Set the meeting link.
    pub fn with_meeting_link(mut self, link: String) -> Self {
        self.meeting_link = Some(link);
        self
    }

    /// Add attendees.
    pub fn with_attendees(mut self, attendees: Vec<String>) -> Self {
        self.attendees = attendees;
        self
    }
}

/// Result of creating a calendar event.
#[derive(Debug, Clone)]
pub struct CalendarEventResult {
    /// Event ID from Google Calendar
    pub event_id: String,
    /// Link to the event in Google Calendar
    pub html_link: String,
}
