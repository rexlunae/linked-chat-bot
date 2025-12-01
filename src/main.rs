//! LinkedIn Chat Bot - An AI-powered tool to automatically respond to recruiters.
//!
//! This tool uses browser automation to interact with LinkedIn messaging,
//! AI to generate professional responses, and exports interview events
//! to Google Calendar.

mod ai;
mod calendar;
mod config;
mod linkedin;

use anyhow::{Context, Result};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::ai::AIAssistant;
use crate::calendar::{CalendarClient, InterviewEvent};
use crate::config::Config;
use crate::linkedin::LinkedInClient;

/// LinkedIn Chat Bot - Automatically respond to recruiters and schedule interviews.
#[derive(Parser)]
#[command(name = "linked-chat-bot")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to configuration file (optional, defaults to environment variables)
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Verbose mode (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the bot to monitor and respond to messages
    Run {
        /// Check interval in seconds
        #[arg(short, long, default_value = "60")]
        interval: u64,

        /// Maximum number of messages to process per run
        #[arg(short, long, default_value = "10")]
        max_messages: usize,

        /// Dry run mode - don't actually send messages
        #[arg(long)]
        dry_run: bool,
    },

    /// Check for new messages without responding
    Check {
        /// Show full conversation history
        #[arg(short, long)]
        full: bool,
    },

    /// Send a test message response
    Test {
        /// Message to respond to (for testing)
        message: String,
    },

    /// View your LinkedIn profile
    Profile,

    /// List upcoming interviews from Google Calendar
    Interviews {
        /// Maximum number of events to show
        #[arg(short, long, default_value = "10")]
        limit: i32,
    },

    /// Export an interview to Google Calendar
    Export {
        /// Job title
        #[arg(long)]
        title: String,

        /// Company name
        #[arg(long)]
        company: String,

        /// Interview date (YYYY-MM-DD)
        #[arg(long)]
        date: String,

        /// Interview time (HH:MM)
        #[arg(long)]
        time: String,

        /// Duration in minutes
        #[arg(long, default_value = "60")]
        duration: u32,

        /// Interview type (phone, video, onsite)
        #[arg(long, default_value = "video")]
        interview_type: String,

        /// Meeting link
        #[arg(long)]
        meeting_link: Option<String>,

        /// Location
        #[arg(long)]
        location: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up logging
    let log_level = match cli.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| format!("linked_chat_bot={log_level}")),
        ))
        .init();

    // Load configuration
    let config = match &cli.config {
        Some(path) => Config::from_file(path)?,
        None => Config::from_env()?,
    };

    // Execute the command
    match cli.command {
        Commands::Run {
            interval,
            max_messages,
            dry_run,
        } => run_bot(&config, interval, max_messages, dry_run).await,
        Commands::Check { full } => check_messages(&config, full).await,
        Commands::Test { message } => test_response(&config, &message).await,
        Commands::Profile => show_profile(&config).await,
        Commands::Interviews { limit } => list_interviews(&config, limit).await,
        Commands::Export {
            title,
            company,
            date,
            time,
            duration,
            interview_type,
            meeting_link,
            location,
        } => {
            export_interview(
                &config,
                &title,
                &company,
                &date,
                &time,
                duration,
                &interview_type,
                meeting_link.as_deref(),
                location.as_deref(),
            )
            .await
        }
    }
}

/// Run the bot to monitor and respond to messages.
async fn run_bot(
    config: &Config,
    interval: u64,
    max_messages: usize,
    dry_run: bool,
) -> Result<()> {
    info!(
        "Starting LinkedIn Chat Bot (interval: {}s, max_messages: {}, dry_run: {})",
        interval, max_messages, dry_run
    );

    // Initialize LinkedIn client
    let mut linkedin = LinkedInClient::new().context("Failed to create LinkedIn client")?;

    // Log in to LinkedIn
    linkedin
        .login(&config.linkedin)
        .context("Failed to log in to LinkedIn")?;

    // Get user's LinkedIn profile for context
    let linkedin_profile = linkedin
        .get_my_profile()
        .context("Failed to get LinkedIn profile")?;

    let profile_summary = linkedin_profile.to_summary();
    info!("Logged in as: {}", linkedin_profile.name);

    // Initialize AI assistant
    let ai = AIAssistant::new(&config.openai.api_key, &config.openai.model);

    // Initialize Google Calendar client (optional)
    let calendar = match CalendarClient::new(&config.google_calendar).await {
        Ok(client) => Some(client),
        Err(e) => {
            warn!(
                "Failed to initialize Google Calendar client: {}. Interview export will be disabled.",
                e
            );
            None
        }
    };

    // Main loop
    loop {
        info!("Checking for new messages...");

        match linkedin.get_conversations() {
            Ok(conversations) => {
                let unread: Vec<_> = conversations
                    .into_iter()
                    .filter(|c| c.has_unread)
                    .take(max_messages)
                    .collect();

                if unread.is_empty() {
                    info!("No unread messages found");
                } else {
                    info!("Found {} unread conversations", unread.len());

                    for conversation in unread {
                        process_conversation(
                            &linkedin,
                            &ai,
                            &calendar,
                            &conversation,
                            &config.user_profile,
                            Some(&profile_summary),
                            dry_run,
                        )
                        .await?;
                    }
                }
            }
            Err(e) => {
                error!("Failed to fetch conversations: {}", e);
            }
        }

        info!("Sleeping for {} seconds...", interval);
        tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
    }
}

/// Process a single conversation.
async fn process_conversation(
    linkedin: &LinkedInClient,
    ai: &AIAssistant,
    calendar: &Option<CalendarClient>,
    conversation: &linkedin::Conversation,
    user_profile: &config::UserProfile,
    linkedin_summary: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    info!(
        "Processing conversation with: {}",
        conversation.participants.join(", ")
    );

    // Generate AI response
    let response = ai
        .generate_response(conversation, user_profile, linkedin_summary)
        .await
        .context("Failed to generate AI response")?;

    info!("Generated response: {}", response.message);

    // Handle extracted job opportunity
    if let Some(job) = &response.job_opportunity {
        info!(
            "Extracted job opportunity: {} at {}",
            job.title, job.company
        );
    }

    // Handle interview scheduling
    if let Some(interview) = &response.interview {
        info!(
            "Interview to schedule: {} at {} on {} {}",
            interview.interview_type, interview.company, interview.date, interview.time
        );

        if let Some(cal) = calendar {
            // Parse the date and time
            if let Ok(event) = create_interview_event(interview) {
                match cal.create_interview_event(&event).await {
                    Ok(result) => {
                        info!(
                            "Created calendar event: {} ({})",
                            result.event_id, result.html_link
                        );
                    }
                    Err(e) => {
                        error!("Failed to create calendar event: {}", e);
                    }
                }
            }
        }
    }

    // Send the response (unless dry run)
    if !dry_run && !response.message.is_empty() {
        linkedin
            .send_message(&conversation.id, &response.message)
            .context("Failed to send message")?;
        info!("Message sent successfully");
    } else if dry_run {
        info!("[DRY RUN] Would send message: {}", response.message);
    }

    Ok(())
}

/// Create an interview event from parsed interview data.
fn create_interview_event(interview: &ai::ParsedInterview) -> Result<InterviewEvent> {
    // Parse date and time
    let date = NaiveDate::parse_from_str(&interview.date, "%Y-%m-%d")
        .context("Failed to parse interview date")?;
    let time = NaiveTime::parse_from_str(&interview.time, "%H:%M")
        .context("Failed to parse interview time")?;
    let datetime = NaiveDateTime::new(date, time);

    // Convert to UTC DateTime
    let start_time = datetime.and_utc();

    let title = format!(
        "{} Interview - {} at {}",
        interview.interview_type, interview.job_title, interview.company
    );

    let description = format!(
        "Job: {} at {}\nType: {}\n\nInterviewers: {}\n\nNotes: {}",
        interview.job_title,
        interview.company,
        interview.interview_type,
        interview.interviewers.join(", "),
        interview.notes.as_deref().unwrap_or("None")
    );

    let mut event =
        InterviewEvent::new(title, description, start_time, interview.duration_minutes);

    if let Some(link) = &interview.meeting_link {
        event = event.with_meeting_link(link.clone());
    }

    if let Some(location) = &interview.location {
        event = event.with_location(location.clone());
    }

    event = event.with_attendees(interview.interviewers.clone());

    Ok(event)
}

/// Check for new messages without responding.
async fn check_messages(config: &Config, full: bool) -> Result<()> {
    info!("Checking LinkedIn messages");

    let mut linkedin = LinkedInClient::new().context("Failed to create LinkedIn client")?;

    linkedin
        .login(&config.linkedin)
        .context("Failed to log in to LinkedIn")?;

    let conversations = linkedin
        .get_conversations()
        .context("Failed to fetch conversations")?;

    if conversations.is_empty() {
        println!("No conversations found");
        return Ok(());
    }

    println!("Found {} conversations:", conversations.len());
    println!();

    for conv in conversations {
        let unread_indicator = if conv.has_unread { " [UNREAD]" } else { "" };
        println!(
            "- {} ({}){unread_indicator}",
            conv.participants.join(", "),
            conv.id
        );

        if full && !conv.messages.is_empty() {
            println!("  Messages:");
            for msg in &conv.messages {
                println!("    [{}] {}: {}", msg.timestamp, msg.sender_name, msg.content);
            }
            println!();
        }
    }

    Ok(())
}

/// Test AI response generation.
async fn test_response(config: &Config, message: &str) -> Result<()> {
    info!("Testing AI response for message: {}", message);

    let ai = AIAssistant::new(&config.openai.api_key, &config.openai.model);

    // Create a mock conversation
    let mut conversation =
        linkedin::Conversation::new("test-conv".to_string(), vec!["Test Recruiter".to_string()]);

    conversation.messages.push(linkedin::Message {
        id: "test-msg".to_string(),
        content: message.to_string(),
        sender_name: "Test Recruiter".to_string(),
        sender_id: "test-recruiter".to_string(),
        direction: linkedin::MessageDirection::Received,
        timestamp: chrono::Utc::now(),
        is_read: false,
    });

    let response = ai
        .generate_response(&conversation, &config.user_profile, None)
        .await
        .context("Failed to generate AI response")?;

    println!("AI Response:");
    println!("{}", response.message);

    if let Some(job) = &response.job_opportunity {
        println!("\nExtracted Job Opportunity:");
        println!("  Title: {}", job.title);
        println!("  Company: {}", job.company);
        if let Some(location) = &job.location {
            println!("  Location: {location}");
        }
        if let Some(salary) = &job.salary_range {
            println!("  Salary: {salary}");
        }
    }

    if let Some(interview) = &response.interview {
        println!("\nInterview to Schedule:");
        println!("  Company: {}", interview.company);
        println!("  Type: {}", interview.interview_type);
        println!("  Date: {}", interview.date);
        println!("  Time: {}", interview.time);
    }

    Ok(())
}

/// Show the user's LinkedIn profile.
async fn show_profile(config: &Config) -> Result<()> {
    info!("Fetching LinkedIn profile");

    let mut linkedin = LinkedInClient::new().context("Failed to create LinkedIn client")?;

    linkedin
        .login(&config.linkedin)
        .context("Failed to log in to LinkedIn")?;

    let profile = linkedin
        .get_my_profile()
        .context("Failed to get LinkedIn profile")?;

    println!("LinkedIn Profile:");
    println!("  Name: {}", profile.name);

    if let Some(headline) = &profile.headline {
        println!("  Headline: {headline}");
    }

    if let Some(location) = &profile.location {
        println!("  Location: {location}");
    }

    println!("  Profile URL: {}", profile.profile_url);

    if !profile.skills.is_empty() {
        println!("  Skills: {}", profile.skills.join(", "));
    }

    Ok(())
}

/// List upcoming interviews from Google Calendar.
async fn list_interviews(config: &Config, limit: i32) -> Result<()> {
    info!("Listing upcoming interviews");

    let calendar = CalendarClient::new(&config.google_calendar)
        .await
        .context("Failed to create Google Calendar client")?;

    let events = calendar
        .list_upcoming_events(limit)
        .await
        .context("Failed to list calendar events")?;

    if events.is_empty() {
        println!("No upcoming events found");
        return Ok(());
    }

    println!("Upcoming events ({}):", events.len());
    println!();

    for event in events {
        let title = event.summary.unwrap_or_else(|| "(No title)".to_string());
        let start = event
            .start
            .and_then(|s| s.date_time)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| "(No time)".to_string());

        println!("- {} at {}", title, start);

        if let Some(location) = event.location {
            println!("  Location: {location}");
        }

        if let Some(link) = event.html_link {
            println!("  Link: {link}");
        }

        println!();
    }

    Ok(())
}

/// Export an interview to Google Calendar.
async fn export_interview(
    config: &Config,
    title: &str,
    company: &str,
    date: &str,
    time: &str,
    duration: u32,
    interview_type: &str,
    meeting_link: Option<&str>,
    location: Option<&str>,
) -> Result<()> {
    info!("Exporting interview to Google Calendar");

    // Parse date and time
    let date = NaiveDate::parse_from_str(date, "%Y-%m-%d").context("Invalid date format")?;
    let time = NaiveTime::parse_from_str(time, "%H:%M").context("Invalid time format")?;
    let datetime = NaiveDateTime::new(date, time);
    let start_time = datetime.and_utc();

    let event_title = format!("{interview_type} Interview - {title} at {company}");
    let description = format!("Job: {title} at {company}\nType: {interview_type}");

    let mut event = InterviewEvent::new(event_title, description, start_time, duration);

    if let Some(link) = meeting_link {
        event = event.with_meeting_link(link.to_string());
    }

    if let Some(loc) = location {
        event = event.with_location(loc.to_string());
    }

    let calendar = CalendarClient::new(&config.google_calendar)
        .await
        .context("Failed to create Google Calendar client")?;

    let result = calendar
        .create_interview_event(&event)
        .await
        .context("Failed to create calendar event")?;

    println!("Interview exported to Google Calendar!");
    println!("Event ID: {}", result.event_id);
    println!("Link: {}", result.html_link);

    Ok(())
}
