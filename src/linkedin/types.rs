//! LinkedIn data types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a recruiter contact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recruiter {
    /// LinkedIn profile ID
    pub profile_id: String,
    /// Recruiter's name
    pub name: String,
    /// Recruiter's title/role
    pub title: Option<String>,
    /// Company they work for
    pub company: Option<String>,
    /// Profile URL
    pub profile_url: String,
}

/// Represents a job opportunity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobOpportunity {
    /// Unique identifier
    pub id: String,
    /// Job title
    pub title: String,
    /// Company name
    pub company: String,
    /// Job location
    pub location: Option<String>,
    /// Remote/hybrid/onsite
    pub work_type: Option<String>,
    /// Salary range
    pub salary_range: Option<String>,
    /// Job description
    pub description: Option<String>,
    /// Recruiter contact
    pub recruiter: Option<Recruiter>,
    /// When this opportunity was first received
    pub received_at: DateTime<Utc>,
}

/// Interview scheduling details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterviewDetails {
    /// Job opportunity this interview is for
    pub job_id: String,
    /// Interview type (phone, video, onsite, etc.)
    pub interview_type: String,
    /// Scheduled date and time
    pub scheduled_time: DateTime<Utc>,
    /// Duration in minutes
    pub duration_minutes: u32,
    /// Meeting link (for video interviews)
    pub meeting_link: Option<String>,
    /// Interview location (for onsite)
    pub location: Option<String>,
    /// Interviewer names
    pub interviewers: Vec<String>,
    /// Additional notes
    pub notes: Option<String>,
}

impl JobOpportunity {
    /// Create a new job opportunity.
    pub fn new(title: String, company: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            company,
            location: None,
            work_type: None,
            salary_range: None,
            description: None,
            recruiter: None,
            received_at: Utc::now(),
        }
    }
}

impl InterviewDetails {
    /// Create new interview details.
    pub fn new(
        job_id: String,
        interview_type: String,
        scheduled_time: DateTime<Utc>,
        duration_minutes: u32,
    ) -> Self {
        Self {
            job_id,
            interview_type,
            scheduled_time,
            duration_minutes,
            meeting_link: None,
            location: None,
            interviewers: Vec::new(),
            notes: None,
        }
    }
}
