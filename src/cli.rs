//! Command-line interface parser for work schedule prediction system.
//! 
//! MIT License
//! 
//! Copyright (c) 2026 66f94eae
//! 
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//! 
//! The above copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software.
//! 
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//! SOFTWARE.

use std::{fs::File, io::Read};

use chrono::{DateTime, Local, NaiveDateTime, NaiveTime};
use clap::{builder::TypedValueParser, Parser};

use crate::conf::Conf;

/// Help message for date format specification
const HELP_MSG: &str = "Date format must be one of: \"YYYYmmDD\", \"YYYYmmDDHHMMss\" or UNIX timestamp(millisecond)\nLeave empty to use today at 23:59:59";
/// Date format string (YYYYmmDD)
const DATE_FORMAT: &str = "%Y%m%d";
/// Date and time format string (YYYYmmDDHHMMSS)
const DATETIME_FORMAT: &str = "%Y%m%d%H%M%S";

const DEFAULT_DATE_VAUE: &str = "today";

/// Command-line interface structure
#[derive(Parser)]
#[command(
    version(env!("CARGO_PKG_VERSION")),
    author(env!("CARGO_PKG_AUTHORS")),
    about(env!("CARGO_PKG_DESCRIPTION")),
    long_about = "Work schedule prediction tool that analyzes calendar events \
                 to determine work/rest days based on configured patterns."
)]
pub struct Cli {
    /// Target date for schedule analysis
    /// 
    /// Supports multiple formats:
    /// - "today": Use today's date at 23:59:59
    /// - "YYYYmmDD": Specific date (e.g., 20241225 for Christmas 2024)
    /// - "YYYYmmDDHHMMSS": Specific date and time
    /// - UNIX timestamp in millisecond
    #[arg(
        long,
        short,
        required = false,
        value_parser = TimestampParser,
        default_value = DEFAULT_DATE_VAUE,
        help = HELP_MSG
    )]
    date: NaiveDateTime,
    
    /// Configuration file path
    /// 
    /// TOML configuration file containing work/rest patterns,
    /// priority rules, and calendar sources.
    #[arg(
        long,
        short,
        required = true,
        value_parser = ConfParser,
        help = "Path to TOML configuration file"
    )]
    conf: Conf,
}

impl Cli {
    /// Returns a reference to the parsed configuration
    pub fn conf(&self) -> &Conf {
        &self.conf
    }
    
    /// Returns the target date for analysis
    pub fn date(&self) -> NaiveDateTime {
        self.date
    }
}

/// Custom parser for timestamp values
#[derive(Clone)]
struct TimestampParser;

impl TypedValueParser for TimestampParser {
    type Value = NaiveDateTime;

    /// Parses timestamp strings from command-line arguments
    /// 
    /// # Arguments
    /// * `value` - String value from command line
    /// 
    /// # Returns
    /// * `Result<NaiveDateTime, clap::Error>` - Parsed datetime or error
    /// 
    /// # Supported Formats
    /// * "today": Today's date at 23:59:59
    /// * "YYYYmmDD": Date only (e.g., 20241225)
    /// * "YYYYmmDDHHMMSS": Full timestamp (e.g., 20241225143000)
    /// * UNIX timestamp in millisecond
    fn parse_ref(
        &self,
        _cmd: &clap::Command,
        _arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let Some(value_str) = value.to_str() else {
            return Err(clap::Error::new(clap::error::ErrorKind::DisplayHelp));
        };
        
        match value_str {
            DEFAULT_DATE_VAUE => {
                // Default to today at 23:59:59
                let end_of_day = NaiveTime::from_hms_opt(23, 59, 59)
                    .ok_or_else(|| clap::Error::raw(
                        clap::error::ErrorKind::InvalidValue,
                        "Failed to create today time 23:59:59"
                    ))?;
                
                match Local::now().with_time(end_of_day) {
                    chrono::offset::LocalResult::Single(dt) => Ok(dt.naive_local()),
                    chrono::offset::LocalResult::Ambiguous(_, later) => Ok(later.naive_local()),
                    chrono::offset::LocalResult::None => Err(clap::Error::raw(
                        clap::error::ErrorKind::InvalidValue,
                        "Invalid date/time combination"
                    )),
                }
            },
            _ => {
                // Try parsing as date only first (YYYYmmDD)
                if let Ok(dt) = NaiveDateTime::parse_from_str(value_str, DATE_FORMAT) {
                    return Ok(dt);
                }
                
                // Try parsing as full timestamp (YYYYmmDDHHMMSS)
                if let Ok(dt) = NaiveDateTime::parse_from_str(value_str, DATETIME_FORMAT) {
                    return Ok(dt);
                }

                // Try parsing as unix timestamp
                if let Ok(time_stamp) = value_str.parse::<i64>() {
                    if let Some(dt) = DateTime::from_timestamp_millis(time_stamp) {
                        return Ok(dt.naive_local())
                    }

                }
                
                // Both formats failed
                Err(clap::Error::raw(
                    clap::error::ErrorKind::InvalidValue,
                    HELP_MSG
                ))
            }
        }
    }
}

/// Custom parser for configuration file loading
#[derive(Clone)]
struct ConfParser;

impl TypedValueParser for ConfParser {
    type Value = Conf;

    /// Parses configuration file path and loads the configuration
    /// 
    /// # Arguments
    /// * `value` - Path to configuration file
    /// 
    /// # Returns
    /// * `Result<Conf, clap::Error>` - Parsed configuration or error
    /// 
    /// # Errors
    /// * File not found or permission denied
    /// * Invalid TOML format
    /// * Configuration validation failures
    fn parse_ref(
        &self,
        _cmd: &clap::Command,
        _arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let Some(file_path) = value.to_str() else {
            return Err(clap::Error::new(clap::error::ErrorKind::DisplayHelp));
        };
        
        // Open configuration file
        let mut file = File::open(file_path).map_err(|e| {
            let error_msg = match e.kind() {
                std::io::ErrorKind::NotFound => format!("Configuration file '{}' not found", file_path),
                std::io::ErrorKind::PermissionDenied => format!("Permission denied for '{}'", file_path),
                _ => format!("Cannot access configuration file '{}': {}", file_path, e),
            };
            clap::Error::raw(clap::error::ErrorKind::InvalidValue, error_msg)
        })?;
        
        // Read file contents
        let mut config_content = String::new();
        file.read_to_string(&mut config_content).map_err(|e| {
            clap::Error::raw(
                clap::error::ErrorKind::InvalidValue,
                format!("Failed to read configuration file '{}': {}", file_path, e)
            )
        })?;
        
        // Parse TOML configuration
        toml::from_str(&config_content).map_err(|e| {
            clap::Error::raw(
                clap::error::ErrorKind::InvalidValue,
                format!("Invalid configuration in '{}': {}", file_path, e)
            )
        })
    }
}