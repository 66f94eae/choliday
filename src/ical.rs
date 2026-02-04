//! iCalendar parser and analyzer for work schedule prediction.
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

use std::{fs::File, io::{BufRead, BufReader, Cursor}, sync::Arc};

use chrono::{DateTime, Datelike, NaiveDateTime, TimeZone, Timelike};
use chrono_tz::Tz;
use ical::property::Property;

use crate::conf::{Conf, Priority};

/// iCalendar property key for event summary
const KEY_SUMMARY: &str = "SUMMARY";
/// iCalendar property key for event start time
const KEY_DTSTART: &str = "DTSTART";
/// iCalendar property key for event end time
const KEY_DTEND: &str = "DTEND";
/// iCalendar property key for event description
const KEY_DESCRIPTION: &str = "DESCRIPTION";

/// iCalendar datetime format: YYYYMMDDTHHMMSS
const DT_FMT: &str = "%Y%m%dT%H%M%S";

/// Day type classification based on calendar events
#[derive(PartialEq, Debug)]
pub enum DayType {
    /// No relevant events found
    NormalDay,
    /// Day classified as work day
    WorkDay,
    /// Day classified as rest day
    RestDay,
    /// Conflicting classifications (both work and rest indicators found)
    ConflictDay,
}

/// Main iCalendar parser and analyzer
pub struct Ical {
    /// Configuration for prediction and analysis
    conf: Conf,
    /// Parsed calendar events (cached after first read)
    events: Option<Vec<Event>>,
}

/// Individual calendar event representation
#[derive(Clone)]
struct Event {
    /// Event title/summary
    summary: String,
    /// Optional event description
    description: Option<String>,
    /// Start timestamp in milliseconds since Unix epoch
    dtstart: i64,
    /// End timestamp in milliseconds since Unix epoch
    dtend: i64,
}

impl Event {
    /// Creates a new empty event
    fn new() -> Self {
        Event {
            summary: "".to_string(),
            description: None,
            dtstart: 0,
            dtend: 0,
        }
    }

    /// Sets the event summary/title
    fn set_summary(&mut self, summary: &str) {
        self.summary = summary.to_string();
    }
    
    /// Returns a reference to the event summary
    fn summary(&self) -> &str {
        &self.summary
    }

    /// Returns the event description if available
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    
    /// Sets the event description
    pub fn set_description(&mut self, desc: impl Into<String>) {
        self.description = Some(desc.into());
    }

    /// Sets the event start timestamp (milliseconds since Unix epoch)
    pub fn set_dtstart(&mut self, dtstart: i64) {
        self.dtstart = dtstart;
    }
    
    /// Sets the event end timestamp (milliseconds since Unix epoch)
    pub fn set_dtend(&mut self, dtend: i64) {
        self.dtend = dtend;
    }
    
    /// Checks if a specific timestamp falls within this event's timeframe
    /// 
    /// # Arguments
    /// * `dt` - Timestamp in milliseconds to check
    /// 
    /// # Returns
    /// * `true` if timestamp is within [dtstart, dtend)
    /// * `false` otherwise
    pub fn contains_timestamp(&self, dt: i64) -> bool {
        self.dtstart <= dt && dt < self.dtend
    }

    /// Classifies the event day type based on prediction patterns
    /// 
    /// # Arguments
    /// * `predict_work` - Patterns indicating work days
    /// * `predict_rest` - Patterns indicating rest days
    /// 
    /// # Returns
    /// * `DayType` classification based on event content
    pub fn day_type(&self, predict_work: &[String], predict_rest: &[String]) -> DayType {
        let summary = self.summary();

        // Check summary for work/rest patterns
        let work_day_summary = predict_work.iter().any(|x| summary.contains(x));
        let rest_day_summary = predict_rest.iter().any(|x| summary.contains(x));

        match (work_day_summary, rest_day_summary) {
            (true, true) => return DayType::ConflictDay,
            (true, false) => return DayType::WorkDay,
            (false, true) => return DayType::RestDay,
            (false, false) => (),
        }

        // If summary didn't match, check description
        if let Some(description) = self.description() {
            let work_day_description = predict_work.iter().any(|x| description.contains(x));
            let rest_day_description = predict_rest.iter().any(|x| description.contains(x));

            match (work_day_description, rest_day_description) {
                (true, true) => DayType::ConflictDay,
                (true, false) => DayType::WorkDay,
                (false, true) => DayType::RestDay,
                (false, false) => DayType::NormalDay,
            }
        } else {
            DayType::NormalDay
        }
    }
}

impl Ical {
    /// Creates a new iCalendar analyzer with the given configuration
    pub fn new(conf: Conf) -> Self {
        Ical {
            conf,
            events: None,
        }
    }

    /// Reads and parses calendar events from configured sources
    /// 
    /// # Arguments
    /// * `dest_day_ts` - Optional timestamp to filter events for a specific day
    /// 
    /// # Note
    /// Supports both HTTP URLs and local file paths
    pub async fn read_events(&mut self, dest_day_ts: Option<i64>) {
        let client = reqwest::Client::new();
        let client = Arc::new(client);

        let tasks = self.conf
            .get_describe_calendar()
            .unwrap_or_else(|| &[])
            .iter()
            .map(|uri| {
                let client = Arc::clone(&client);
                async move {
                    if uri.starts_with("http") {
                        // Fetch from remote URL
                        if let Ok(resp) = client.get(uri).send().await {
                            if let Ok(bytes) = resp.bytes().await {
                                return Self::parse_calendar(Cursor::new(bytes), dest_day_ts);
                            }
                        }
                        Vec::new()
                    } else {
                        // Read from local file
                        if let Ok(file) = File::open(uri) {
                            return Self::parse_calendar(BufReader::new(file), dest_day_ts);
                        }
                        Vec::new()
                    }
                }
            });

        let mut all_events = Vec::new();
        for task in tasks {
            let events = task.await;
            all_events.extend(events);
        }

        self.events = Some(all_events);
    }

    /// Determines the day type by applying priority rules to calendar events
    /// 
    /// # Arguments
    /// * `dest_day_ts` - Optional timestamp for specific day analysis
    /// 
    /// # Returns
    /// * `DayType` based on priority rules and event analysis
    pub async fn judge_by_priority(&mut self, dest_day_ts: Option<i64>) -> DayType {
        // Ensure events are loaded
        if self.events.is_none() {
            self.read_events(dest_day_ts).await;
        }
        
        let events = self.events.clone().unwrap();
        let predict_work = self.conf.predict_work();
        let predict_rest = self.conf.predict_rest();
        
        // Filter and classify events
        let day_types: Vec<DayType> = match dest_day_ts {
            Some(dts) => events
                .iter()
                .filter(|e| e.contains_timestamp(dts))
                .map(|e| e.day_type(predict_work, predict_rest))
                .filter(|x| *x != DayType::NormalDay)
                .collect(),
            None => events
                .iter()
                .map(|e| e.day_type(predict_work, predict_rest))
                .filter(|x| *x != DayType::NormalDay)
                .collect(),
        };
        
        if day_types.is_empty() {
            return DayType::NormalDay;
        }

        match self.conf.priority() {
            Priority::WorkOverRest => {
                if day_types.iter().any(|x| *x == DayType::WorkDay || *x == DayType::ConflictDay) {
                    DayType::WorkDay
                } else {
                    DayType::RestDay
                }
            },
            Priority::RestOverWork => {
                if day_types.iter().any(|x| *x == DayType::RestDay || *x == DayType::ConflictDay) {
                    DayType::RestDay
                } else {
                    DayType::WorkDay
                }
            },
            Priority::KeepCurrent => {
                match day_types.first() {
                    Some(DayType::RestDay) => DayType::RestDay,
                    _ => DayType::WorkDay,
                }
            },
            Priority::UseLatest => {
                match day_types.last() {
                    Some(DayType::RestDay) => DayType::RestDay,
                    _ => DayType::WorkDay,
                }
            },
        }
    }

    /// Parses iCalendar data from a reader
    /// 
    /// # Arguments
    /// * `reader` - Buffered reader containing iCalendar data
    /// * `filter` - Optional timestamp to filter events
    /// 
    /// # Returns
    /// * `Vec<Event>` - Parsed events
    fn parse_calendar<T: BufRead>(reader: T, filter: Option<i64>) -> Vec<Event> {
        let mut events = Vec::new();
        let parser = ical::IcalParser::new(reader);
        
        for calendar in parser {
            if let Ok(cal) = calendar {
                for event in cal.events {
                    let mut my_event = Event::new();
                    
                    for prop in event.properties {
                        match prop.name.as_str() {
                            KEY_SUMMARY => {
                                if let Some(summary) = prop.value {
                                    my_event.set_summary(&summary);
                                } else {
                                    my_event.set_summary("NO_SUMMARY");
                                }
                            },
                            KEY_DESCRIPTION => {
                                if let Some(desc) = prop.value {
                                    my_event.set_description(desc);
                                }
                            },
                            KEY_DTSTART => {
                                if let Ok(timestamp) = Self::parse_datetime(&prop, true) {
                                    my_event.set_dtstart(timestamp);
                                }
                            },
                            KEY_DTEND => {
                                if let Ok(timestamp) = Self::parse_datetime(&prop, false) {
                                    my_event.set_dtend(timestamp);
                                }
                            },
                            _ => {}
                        }
                    }
                    
                    // Handle events with no explicit end time
                    if my_event.dtend == 0 {
                        my_event.dtend = my_event.dtstart;
                    }
                    
                    // Apply filter if specified
                    if let Some(filter_ts) = filter {
                        if my_event.contains_timestamp(filter_ts) {
                            events.push(my_event);
                        }
                    } else {
                        events.push(my_event);
                    }
                }
            }
        }
        events
    }

    /// Parses iCalendar datetime strings into Unix timestamps
    /// 
    /// # Arguments
    /// * `prop` - iCalendar property containing datetime
    /// * `is_dt_start` - Whether this is a DTSTART (true) or DTEND (false)
    /// 
    /// # Returns
    /// * `Result<i64, &str>` - Unix timestamp in milliseconds or error message
    /// 
    /// # Supported Formats
    /// * YYYYMMDD (all-day events)
    /// * YYYYMMDDTHHMMSS (local time)
    /// * YYYYMMDDTHHMMSSZ (UTC time)
    /// * YYYYMMDDTHHMMSS with TZID parameter
    fn parse_datetime(prop: &Property, is_dt_start: bool) -> Result<i64, &'static str> {
        let Some(value) = &prop.value else {
            return Err("Missing datetime value");
        };
        
        let mut value = value.to_uppercase();
        
        match value.len() {
            8 => {
                // All-day event: YYYYMMDD
                let fill_with = if is_dt_start { "T000000" } else { "T235959" };
                value.push_str(fill_with);
                
                NaiveDateTime::parse_from_str(&value, DT_FMT)
                    .map(|dt| dt.and_utc().timestamp_millis())
                    .map_err(|_| "Invalid datetime format")
            },
            _ => {
                if value.ends_with('Z') {
                    // UTC timezone
                    DateTime::parse_from_str(&value, DT_FMT)
                        .map(|dt| dt.timestamp_millis())
                        .map_err(|_| "Invalid datetime format")
                } else {
                    // Check for timezone parameter
                    if let Some(params) = &prop.params {
                        for (name, field) in params {
                            if name.to_uppercase() == "TZID" && !field.is_empty() {
                                if let Ok(tz) = field[0].parse::<Tz>() {
                                    if let Ok(dt) = NaiveDateTime::parse_from_str(&value, DT_FMT) {
                                        let dt_result = tz.with_ymd_and_hms(
                                            dt.year(),
                                            dt.month(),
                                            dt.day(),
                                            dt.hour(),
                                            dt.minute(),
                                            dt.second(),
                                        );
                                        
                                        match dt_result {
                                            chrono::offset::LocalResult::Single(tz_dt) => {
                                                return Ok(tz_dt.timestamp_millis());
                                            },
                                            chrono::offset::LocalResult::Ambiguous(early, later) => {
                                                let tz_dt = if is_dt_start { early } else { later };
                                                return Ok(tz_dt.timestamp_millis());
                                            },
                                            chrono::offset::LocalResult::None => {
                                                return Err("Invalid datetime for timezone");
                                            },
                                        }
                                    }
                                } else {
                                    return Err("Invalid timezone identifier");
                                }
                            }
                        }
                    }
                    
                    // Fallback to UTC if no timezone specified
                    NaiveDateTime::parse_from_str(&value, DT_FMT)
                        .map(|dt| dt.and_utc().timestamp_millis())
                        .map_err(|_| "Invalid datetime format")
                }
            }
        }
    }
}