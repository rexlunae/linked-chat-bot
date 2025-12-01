//! LinkedIn profile data structures.

use serde::{Deserialize, Serialize};

/// User's LinkedIn profile information.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LinkedInProfile {
    /// LinkedIn profile ID
    pub id: String,
    /// User's full name
    pub name: String,
    /// Profile headline
    pub headline: Option<String>,
    /// Current location
    pub location: Option<String>,
    /// About/summary section
    pub summary: Option<String>,
    /// Current position
    pub current_position: Option<Position>,
    /// Work experience
    pub experience: Vec<Position>,
    /// Education
    pub education: Vec<Education>,
    /// Skills
    pub skills: Vec<String>,
    /// Profile URL
    pub profile_url: String,
    /// Contact email (if visible)
    pub email: Option<String>,
    /// Phone number (if visible)
    pub phone: Option<String>,
}

/// Work position/experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Job title
    pub title: String,
    /// Company name
    pub company: String,
    /// Location
    pub location: Option<String>,
    /// Start date (YYYY-MM format)
    pub start_date: Option<String>,
    /// End date (YYYY-MM format, None if current)
    pub end_date: Option<String>,
    /// Description
    pub description: Option<String>,
}

/// Education entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Education {
    /// School/university name
    pub school: String,
    /// Degree
    pub degree: Option<String>,
    /// Field of study
    pub field_of_study: Option<String>,
    /// Start year
    pub start_year: Option<u32>,
    /// End year
    pub end_year: Option<u32>,
}

impl LinkedInProfile {
    /// Generate a summary of the profile for AI context.
    pub fn to_summary(&self) -> String {
        let mut summary = format!("Name: {}\n", self.name);

        if let Some(headline) = &self.headline {
            summary.push_str(&format!("Headline: {headline}\n"));
        }

        if let Some(location) = &self.location {
            summary.push_str(&format!("Location: {location}\n"));
        }

        if let Some(current) = &self.current_position {
            summary.push_str(&format!(
                "Current Position: {} at {}\n",
                current.title, current.company
            ));
        }

        if !self.skills.is_empty() {
            summary.push_str(&format!("Key Skills: {}\n", self.skills.join(", ")));
        }

        if let Some(about) = &self.summary {
            summary.push_str(&format!("\nAbout: {about}\n"));
        }

        summary
    }
}
