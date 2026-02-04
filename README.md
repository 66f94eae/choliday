# 🗓️ Choliday - Intelligent Workday Determination Tool

**Choliday** is an intelligent workday determination tool written in Rust. 

It can accurately determine whether any given date is a workday or rest day based on *calendar events*, *custom rules*, and priority settings. 

It is particularly suitable for handling complex scenarios such as :
- If you are working day, send notification about weather
- If you are resting day, close all alarm
- Share your busy status with someone

---

## ✨ Core Features

- 🎯 **Multiple Calendar Source Support** - Supports local and remote iCal calendars
- 🔧 **Flexible Configuration** - Define all rules through TOML configuration files
- 🔄 **Priority System** - Intelligent decision-making for conflicting rules
- 📊 **Pattern Matching** - Keyword matching for calendar events
- 🚀 **High Performance** - Asynchronous processing, fast response
- 📝 **Script-Friendly** - Suitable for integration into automated workflows

---

## 🚀 Quick Start

### Installation

```bash
# Install from source
cargo install --path .

# Or run directly with Cargo
cargo run -- -c config.toml
```
### Basic Usage
```bash

# Check if today is a workday (uses default time 23:59:59)
choliday -c config.toml

# Check a specific date
choliday -c config.toml -d 20241225
# date with time
choliday -c config.toml -d 20241225143000
# timestamp(millisecond)
choliday -c config.toml -d 1735108200000

# View help
choliday --help
```


### ⚙️ Detailed Judgment Logic

#### 1. Three-Layer Judgment System
Calendar Event Matching → Workday Rules → Default Weekend Rules

#### 2. Priority Order
1. **First Priority: Calendar Event Matching**

    Work Event Matching: Event title or description contains work keywords → Workday

    Rest Event Matching: Event title or description contains rest keywords → Rest Day

    Conflicting Events: Simultaneously matches both work and rest keywords → Processed according to configured priority

2. **Second Priority: Custom Workday Rules**

    Define weekly workdays in config.toml

    Example: workday = "1-5" means Monday to Friday are workdays

    Format support: "1-5", "1,3,5", "1,3-5"

3. **Third Priority: Default Weekend Rules**

    If no workday rules are configured, Saturday and Sunday default to rest days

### 3. Conflict Resolution Strategies

When calendar events conflict (simultaneously matching both work and rest keywords), they are processed according to the configured priority:
|Priority Mode | Behavior |
|:---|:---|
|WorkOverRest|	Work priority, any work mark results in workday judgment|
|RestOverWork|	Rest priority, any rest mark results in rest day judgment|
|KeepCurrent|	Maintain the state of the first matching result|
|UseLatest|	Use the state of the latest matching result|

### 📁 Configuration File Details
Configuration File Example (config.toml)
```toml

[base]
# Workday definition (1=Monday, 7=Sunday)
workday = "1-5"  # Monday to Friday

[calendar]
# Calendar sources (support local files and remote URLs)
source = [
    "local_calendar.ics",
    "https://example.com/calendar.ics"
]

[predict]
# Work keywords
work = ["work"]

# Rest keywords
rest = ["rest", "holiday"]

# Conflict resolution priority (WorkOverRest, RestOverWork, KeepCurrent, UseLatest)
priority = "WorkOverRest"
```
#### Configuration Items Explained
***[base] Basic Configuration***
>    
>   **workday**: Workday definition, supports multiple formats:
>
>       Range: "1-5" (Monday to Friday)
>       List: "1,3,5" (Monday, Wednesday, Friday)
>       Mixed: "1,3-5" (Monday, Wednesday to Friday)

***[calendar] Calendar Configuration***
>
>    **source**: List of calendar sources, supports:
>
>        Local files: "path/to/calendar.ics"
>
>        Remote URLs: "https://example.com/calendar.ics"
>
>        Supports simultaneous subscription to multiple calendars

***[predict] Prediction Configuration***
>
>    *work*: List of keywords identifying workdays
>
>    *rest*: List of keywords identifying rest days
>
>    *priority*: Conflict resolution strategy, options:
>
>        WorkOverRest: Work priority
>
>        RestOverWork: Rest priority
>
>        KeepCurrent: Maintain current
>
>        UseLatest: Use latest

### 🗓️ Calendar Format Support
Supported iCal Properties
>
>    *SUMMARY*: Event title (used for keyword matching)
>
>    *DESCRIPTION*: Event description (used for keyword matching)
>
>    *DTSTART*: Start time
>
>    *DTEND*: End time

### Time Format Support
>
>    All-day events: YYYYMMDD
>
>    Specific time: YYYYMMDDTHHMMSS
>
>    UTC time: YYYYMMDDTHHMMSSZ
>
>    Timezone time: Time with TZID parameter

### 🔧 Advanced Usage
#### Using in Scripts
```bash

#!/bin/bash

# Check if tomorrow is a workday
tomorrow=$(date -d "tomorrow" +%Y%m%d)
if choliday -c config.toml -d $tomorrow; then
    echo "Workday tomorrow"
    # Send reminders, etc.
else
    echo "Rest day tomorrow"
fi

# Get exit code
choliday -c config.toml -d 20241225
exit_code=$?
if [ $exit_code -eq 0 ]; then
    echo "Workday"
elif [ $exit_code -eq 1 ]; then
    echo "Rest day"
else
    echo "Error occurred"
fi
```

#### Integration into Other Applications

```rust

use choliday::Choliday;
use cli::Cli;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let analyzer = Choliday::new(&cli);
    
    if analyzer.is_work_day().await {
        println!("Workday today");
    } else {
        println!("Rest day today");
    }
}
```

### 📊 Exit Code Explanation
|Exit Code|	Meaning	Description|
|:---|:---|
|0|	Workday	Target date is a workday|
|1|	Rest Day	Target date is a rest day|
|2|	Error	Program execution error|

### 🛠️ Development Guide
#### Project Structure
```bash
src/
├── main.rs          # Program entry point
├── cli.rs           # Command-line argument parsing
├── conf.rs          # Configuration parsing and validation
├── ical.rs          # iCalendar parsing and event processing
└── choliday.rs      # Core judgment logic
```

#### Building and Testing
```bash

# Development build
cargo build

# Release build
cargo build --release


# Code formatting
cargo fmt
```

#### Dependencies

    chrono: Date and time processing

    chrono-tz: Timezone support

    clap: Command-line parsing

    ical: iCalendar parsing

    reqwest: HTTP client (remote calendars)

    serde: Configuration serialization/deserialization

    tokio: Asynchronous runtime

### 📝 Use Case Examples
+ Scenario 1: Corporate Attendance System

    Subscribe to company official calendar

    Automatically handle make-up workday arrangements

    Integrate into attendance statistics system

+ Scenario 2: Personal Schedule Management

    Subscribe to holiday calendars

    Automatically mark workdays/rest days

    Intelligent reminders for important schedules

+ Scenario 3: Automation Scripts

    CI/CD process control

    Scheduled task management

    Data statistics and analysis

### 🤝 Contribution Guide

    Fork the project

    Create a feature branch (git checkout -b feature/AmazingFeature)

    Commit changes (git commit -m 'Add some AmazingFeature')

    Push to branch (git push origin feature/AmazingFeature)

    Open a Pull Request

### 📄 License

This project is open source under the MIT License - see the LICENSE file for details.

### 🆘 Frequently Asked Questions
**Q1: How to handle cross-timezone issues?**
> A: The tool supports iCal events with timezones and automatically converts them to local time for comparison.

**Q2: How are multiple calendar sources handled?**
> A: The tool merges events from all calendar sources and processes them according to unified rules.

**Q3: Is keyword matching case-sensitive?**
> A: No, all matching is case-insensitive.

**Q4: How to update calendars?**
> A: Remote calendars are re-fetched each time the tool runs to ensure information is up-to-date.


Let Choliday intelligently manage your work calendar and say goodbye to complex make-up workday calculations! 🎉