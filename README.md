# LinkedIn Chat Bot

A Rust command-line agentic tool that automatically responds to LinkedIn recruiters using AI. It can use your LinkedIn profile information to respond to messages, set up interviews, collect job details, and export interview events to Google Calendar.

## Features

- **LinkedIn Integration**: Logs into LinkedIn using browser automation and monitors messages from recruiters
- **AI-Powered Responses**: Uses OpenAI GPT models to generate professional responses to recruiter messages
- **Job Opportunity Tracking**: Automatically extracts and tracks job opportunity details from conversations
- **Interview Scheduling**: Identifies interview scheduling requests and collects relevant details
- **Google Calendar Export**: Exports scheduled interviews to your Google Calendar

## Installation

### Prerequisites

- Rust 1.70 or later
- Chrome/Chromium browser (for headless browser automation)
- OpenAI API key
- Google Calendar API credentials (optional, for calendar integration)

### Building from Source

```bash
git clone https://github.com/rexlunae/linked-chat-bot.git
cd linked-chat-bot
cargo build --release
```

## Configuration

The bot can be configured using environment variables or a configuration file.

### Environment Variables

Create a `.env` file in the project root with the following variables:

```env
# LinkedIn credentials
LINKEDIN_EMAIL=your-email@example.com
LINKEDIN_PASSWORD=your-linkedin-password

# OpenAI configuration
OPENAI_API_KEY=sk-your-api-key-here
OPENAI_MODEL=gpt-4  # Optional, defaults to gpt-4

# Google Calendar (optional)
GOOGLE_CREDENTIALS_PATH=google_credentials.json
GOOGLE_CALENDAR_ID=primary  # Optional, defaults to primary

# User profile information (for AI context)
USER_NAME=Your Name
USER_EMAIL=your-email@example.com
USER_PHONE=+1-555-555-5555  # Optional
USER_LOCATION=San Francisco, CA  # Optional
USER_AVAILABILITY=Weekdays 9am-5pm PST  # Optional
USER_CURRENT_ROLE=Senior Software Engineer  # Optional
USER_TARGET_ROLE=Staff Engineer  # Optional
USER_SALARY_EXPECTATIONS=$200k-$250k  # Optional
USER_NOTES=Looking for remote-first companies  # Optional
```

### Configuration File

Alternatively, you can use a JSON configuration file:

```json
{
  "linkedin": {
    "email": "your-email@example.com",
    "password": "your-linkedin-password"
  },
  "openai": {
    "api_key": "sk-your-api-key-here",
    "model": "gpt-4"
  },
  "google_calendar": {
    "credentials_path": "google_credentials.json",
    "calendar_id": "primary"
  },
  "user_profile": {
    "name": "Your Name",
    "email": "your-email@example.com",
    "phone": "+1-555-555-5555",
    "location": "San Francisco, CA",
    "availability": "Weekdays 9am-5pm PST",
    "current_role": "Senior Software Engineer",
    "target_role": "Staff Engineer",
    "salary_expectations": "$200k-$250k",
    "notes": "Looking for remote-first companies"
  }
}
```

## Usage

### Run the Bot

Start the bot to continuously monitor and respond to LinkedIn messages:

```bash
# Using environment variables
linked-chat-bot run

# With custom interval (check every 2 minutes)
linked-chat-bot run --interval 120

# Dry run mode (don't actually send messages)
linked-chat-bot run --dry-run

# Using a config file
linked-chat-bot --config config.json run
```

### Check Messages

Check for new messages without responding:

```bash
linked-chat-bot check

# Show full conversation history
linked-chat-bot check --full
```

### Test AI Response

Test how the AI would respond to a message:

```bash
linked-chat-bot test "Hi! I'm a recruiter at TechCorp. We have a Senior Engineer position that might interest you. Would you like to learn more?"
```

### View Profile

View your LinkedIn profile as seen by the bot:

```bash
linked-chat-bot profile
```

### List Interviews

List upcoming interviews from Google Calendar:

```bash
linked-chat-bot interviews

# Limit results
linked-chat-bot interviews --limit 5
```

### Export Interview

Manually export an interview to Google Calendar:

```bash
linked-chat-bot export \
  --title "Software Engineer" \
  --company "TechCorp" \
  --date "2024-12-15" \
  --time "10:00" \
  --duration 60 \
  --interview-type video \
  --meeting-link "https://zoom.us/j/123456789"
```

## Google Calendar Setup

To enable Google Calendar integration:

1. Go to the [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select an existing one
3. Enable the Google Calendar API
4. Create OAuth 2.0 credentials (Desktop application)
5. Download the credentials JSON file and save it as `google_credentials.json`
6. The first time you run the bot with calendar features, it will open a browser for OAuth authentication

## How It Works

1. **Login**: The bot uses headless Chrome to log into LinkedIn with your credentials
2. **Monitor**: It periodically checks your LinkedIn messages for unread conversations
3. **Analyze**: For each unread message, the AI analyzes the conversation context
4. **Respond**: The AI generates an appropriate professional response
5. **Extract**: Job opportunities and interview details are automatically extracted
6. **Schedule**: Interview events are exported to Google Calendar

## Security Considerations

- Your LinkedIn and API credentials are stored locally and never transmitted to third parties
- The bot uses headless browser automation, which may trigger LinkedIn security checks
- Consider using app-specific passwords or OAuth tokens where possible
- Review AI-generated responses before enabling auto-send in production

## License

MIT License - see LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
