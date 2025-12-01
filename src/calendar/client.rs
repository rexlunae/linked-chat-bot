//! Google Calendar API client.

use anyhow::{Context, Result};
use google_calendar3::api::{Event, EventDateTime};
use google_calendar3::hyper_rustls::HttpsConnector;
use google_calendar3::{hyper_util, CalendarHub};
use tracing::{debug, info};

use super::types::{CalendarEventResult, InterviewEvent};
use crate::config::GoogleCalendarConfig;

/// Google Calendar API client.
pub struct CalendarClient {
    hub: CalendarHub<HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>>,
    calendar_id: String,
}

impl CalendarClient {
    /// Create a new Calendar client with OAuth2 authentication.
    pub async fn new(config: &GoogleCalendarConfig) -> Result<Self> {
        info!("Initializing Google Calendar client");

        let secret = google_calendar3::yup_oauth2::read_application_secret(&config.credentials_path)
            .await
            .with_context(|| {
                format!(
                    "Failed to read Google credentials from: {}",
                    config.credentials_path.display()
                )
            })?;

        // Set up the authenticator with installed app flow
        let auth = google_calendar3::yup_oauth2::InstalledFlowAuthenticator::builder(
            secret,
            google_calendar3::yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
        )
        .persist_tokens_to_disk("google_token_cache.json")
        .build()
        .await
        .context("Failed to build authenticator")?;

        // Create the HTTP client using hyper_util
        let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
            .build(
                google_calendar3::hyper_rustls::HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .context("Failed to load native root certificates")?
                    .https_or_http()
                    .enable_http1()
                    .build(),
            );

        let hub = CalendarHub::new(client, auth);

        Ok(Self {
            hub,
            calendar_id: config.calendar_id.clone(),
        })
    }

    /// Create an interview event in Google Calendar.
    pub async fn create_interview_event(
        &self,
        interview: &InterviewEvent,
    ) -> Result<CalendarEventResult> {
        info!("Creating calendar event: {}", interview.title);

        // Set start time
        let start = EventDateTime {
            date_time: Some(interview.start_time),
            ..Default::default()
        };

        // Set end time
        let end = EventDateTime {
            date_time: Some(interview.end_time),
            ..Default::default()
        };

        // Build the event
        let location = interview
            .location
            .clone()
            .or_else(|| interview.meeting_link.clone());

        let event = Event {
            summary: Some(interview.title.clone()),
            description: Some(interview.description.clone()),
            start: Some(start),
            end: Some(end),
            location,
            ..Default::default()
        };

        // Insert the event
        let result = self
            .hub
            .events()
            .insert(event, &self.calendar_id)
            .doit()
            .await
            .context("Failed to create calendar event")?;

        let created_event = result.1;

        debug!("Event created with ID: {:?}", created_event.id);

        Ok(CalendarEventResult {
            event_id: created_event.id.unwrap_or_default(),
            html_link: created_event.html_link.unwrap_or_default(),
        })
    }

    /// List upcoming events from the calendar.
    pub async fn list_upcoming_events(&self, max_results: i32) -> Result<Vec<Event>> {
        info!("Listing upcoming events");

        let now = chrono::Utc::now();

        let result = self
            .hub
            .events()
            .list(&self.calendar_id)
            .time_min(now)
            .max_results(max_results)
            .single_events(true)
            .order_by("startTime")
            .doit()
            .await
            .context("Failed to list calendar events")?;

        Ok(result.1.items.unwrap_or_default())
    }

    /// Delete an event from the calendar.
    pub async fn delete_event(&self, event_id: &str) -> Result<()> {
        info!("Deleting calendar event: {}", event_id);

        self.hub
            .events()
            .delete(&self.calendar_id, event_id)
            .doit()
            .await
            .context("Failed to delete calendar event")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_interview_event_creation() {
        let event = InterviewEvent::new(
            "Interview at TechCorp".to_string(),
            "Software Engineer position".to_string(),
            Utc::now(),
            60,
        )
        .with_location("123 Main St".to_string())
        .with_meeting_link("https://zoom.us/j/123456".to_string())
        .with_attendees(vec!["recruiter@example.com".to_string()]);

        assert_eq!(event.title, "Interview at TechCorp");
        assert!(event.location.is_some());
        assert!(event.meeting_link.is_some());
        assert_eq!(event.attendees.len(), 1);
    }
}
