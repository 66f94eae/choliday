//! Configuration module for work schedule prediction system.
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

use std::collections::HashSet;

use serde::{de::{Error, Visitor}, Deserialize};

/// Main configuration structure for the application.
/// 
/// This struct contains all configurable parameters for work schedule prediction,
/// including base settings, calendar sources, and prediction rules.
#[derive(Deserialize, Clone)]
pub struct Conf {
    /// Basic configuration settings
    base: Option<Base>,
    /// Calendar configuration for external data sources
    calendar: Option<Calnedar>,
    /// Prediction rules and patterns
    predict: Predict    
}

/// Basic configuration settings.
/// 
/// Contains fundamental settings like workday definitions.
#[derive(Deserialize, Clone)]
struct Base {
    /// Set of workdays represented as numbers 1-7 (Monday=1 through Sunday=7)
    /// Deserialized from string formats like "1-5", "1,3,5", or "1,3-5"
    #[serde(deserialize_with = "deserialize_workday")]
    workday: HashSet<i8>,
}

/// Prediction configuration for work schedule forecasting.
/// 
/// Defines rules for predicting work and rest days based on patterns.
#[derive(Deserialize, Clone)]
pub struct Predict {
    /// Patterns used to identify work days in schedule prediction
    work: Vec<String>,
    /// Patterns used to identify rest days in schedule prediction
    rest: Vec<String>,
    /// Priority rule for resolving conflicts between work and rest predictions
    priority: Priority
}

/// Priority rules for resolving prediction conflicts.
/// 
/// Determines which prediction takes precedence when conflicts arise.
#[derive(Deserialize, Clone)]
pub enum Priority {
    /// Work predictions override rest predictions
    WorkOverRest,
    /// Rest predictions override work predictions
    RestOverWork,
    /// Keep the current state without change
    KeepCurrent,
    /// Use the most recent prediction
    UseLatest
}

/// Calendar configuration for external data sources.
/// 
/// Allows integration with external calendar systems or data sources.
#[derive(Deserialize, Clone)]
pub struct Calnedar {
    /// List of calendar data source identifiers or URLs
    source: Option<Vec<String>>
}

impl Conf {
    /// Returns the list of calendar data sources if configured.
    /// 
    /// # Returns
    /// - `Some(&[String])`: Reference to calendar source identifiers
    /// - `None`: No calendar sources configured
    pub fn get_describe_calendar(&self) -> Option<&[String]> {
        if let Some(cal) = &self.calendar {
            return cal.source.as_deref();
        }
        None
    }

    /// Returns the priority rule for prediction conflict resolution.
    /// 
    /// # Returns
    /// - Reference to the configured priority rule
    pub fn priority(&self) -> &Priority {
        &self.predict.priority
    }

    /// Returns the work day prediction patterns.
    /// 
    /// # Returns
    /// - Reference to vector of work day patterns
    pub fn predict_work(&self) -> &Vec<String>{
        self.predict.work.as_ref()
    }

    /// Returns the rest day prediction patterns.
    /// 
    /// # Returns
    /// - Reference to vector of rest day patterns
    pub fn predict_rest(&self) -> &Vec<String> {
        self.predict.rest.as_ref()
    }

    /// Returns the set of configured work days.
    /// 
    /// # Returns
    /// - `Some(HashSet<i8>)`: Set of work days (1-7)
    /// - `None`: No work day configuration available
    pub fn work_day(&self) -> Option<HashSet<i8>>{
        self.base.clone().map(|base| base.workday)
    }
}

/// Deserializes workday string into a HashSet of day numbers.
/// 
/// # Arguments
/// * `deserializer` - Serde deserializer instance
/// 
/// # Returns
/// * `Result<HashSet<i8>, D::Error>` - Set of work days or deserialization error
/// 
/// # Supported Formats
/// * Single days: "1", "2", "3"
/// * Day ranges: "1-5"
/// * Mixed formats: "1,3,5" or "1,3-5"
fn deserialize_workday<'de, D>(deserializer: D) -> Result<HashSet<i8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_str(WorkDayVisitor)
}

/// Error message format for workday deserialization errors.
const ERR_FMT: &str = "a workday string like '1-5' or '1,3,5' or '1,3-5' (numbers 1-7 only)";

/// Visitor for deserializing workday strings into HashSet<i8>.
struct WorkDayVisitor;

impl<'a> Visitor<'a> for WorkDayVisitor {
    type Value = HashSet<i8>;

    /// Describes the expected format for error messages.
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "{}", &ERR_FMT)
    }

    /// Processes a string representation of workdays.
    /// 
    /// # Arguments
    /// * `v` - String containing workday specification
    /// 
    /// # Returns
    /// * `Result<HashSet<i8>, E>` - Parsed workday set or error
    /// 
    /// # Examples
    /// * "1-5" → {1, 2, 3, 4, 5}
    /// * "1,3,5" → {1, 3, 5}
    /// * "1,3-5" → {1, 3, 4, 5}
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error, 
    {
        match v.len() {
            0 => Err(Error::invalid_length(0, &ERR_FMT)),
            _ => {
                let mut workday_set: HashSet<i8> = HashSet::new();
                
                // Split by comma to handle multiple day specifications
                for day_spec in v.split(",") {   
                    match day_spec.parse::<i8>() {
                        // Single day number
                        Ok(day) => {
                            let validated_day = weekday_check(day)?;
                            workday_set.insert(validated_day);
                        },
                        // Could be a range or invalid format
                        Err(_) => {
                            let day_range: Vec<&str> = day_spec.split("-").collect();
                            
                            // Must be exactly two parts for a range
                            if day_range.len() != 2 {
                                return Err(Error::invalid_type(
                                    serde::de::Unexpected::Str(day_spec), 
                                    &ERR_FMT
                                ));
                            }
                            
                            // Parse start and end of range
                            match day_range[0].parse::<i8>()
                                .and_then(|a| day_range[1].parse::<i8>().map(|b| (a, b))) 
                            {
                                Ok((start, end)) => {
                                    // Validate both ends of the range
                                    weekday_check(start)?;
                                    weekday_check(end)?;
                                    
                                    // Add all days in the range (inclusive)
                                    workday_set.extend(start.min(end)..=start.max(end));
                                },
                                Err(_) => {
                                    return Err(Error::invalid_type(
                                        serde::de::Unexpected::Str(day_spec), 
                                        &ERR_FMT
                                    ));
                                }
                            }
                        }
                    }
                }
                Ok(workday_set)
            }
        }
    }
}

/// Validates that a day number is within the valid range (1-7).
/// 
/// # Arguments
/// * `x` - Day number to validate
/// 
/// # Returns
/// * `Result<i8, E>` - Validated day number or error
/// 
/// # Note
/// * 1 = Monday, 7 = Sunday (ISO 8601 standard)
fn weekday_check<E>(x: i8) -> Result<i8, E> 
where
    E: serde::de::Error,
{
    if x < 1 || x > 7 {
        return Err(Error::invalid_value(
            serde::de::Unexpected::Signed(x as i64), 
            &"numbers 1-7 only"
        ));
    }
    Ok(x)
}