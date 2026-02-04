//! Holiday and workday determination logic.
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

use std::{cell::RefCell, collections::HashSet};

use chrono::{Datelike, NaiveDateTime};

use crate::{cli::Cli, ical::Ical};

/// Main workday/holiday analyzer
pub struct Choliday {
    /// iCalendar parser and analyzer with interior mutability
    ical: RefCell<Ical>,
    /// Target date for analysis
    dt: NaiveDateTime,
    /// Configured workdays (1-7 where Monday = 1, Sunday = 7)
    /// If None, uses default weekend (Saturday and Sunday)
    work_days: Option<HashSet<i8>>,
}

impl Choliday {
    /// Creates a new holiday analyzer from command-line arguments
    /// 
    /// # Arguments
    /// * `cli` - Parsed command-line arguments containing config and date
    /// 
    /// # Returns
    /// * `Choliday` instance ready for analysis
    pub fn new(cli: &Cli) -> Self {
        let conf = cli.conf();
        Self {
            ical: RefCell::new(Ical::new(conf.clone())),
            dt: cli.date(),
            work_days: conf.work_day(),
        }
    }

    /// Determines if the target date is a workday
    /// 
    /// # Returns
    /// * `true` if the date is a workday
    /// * `false` if the date is a holiday/rest day
    /// 
    /// # Algorithm
    /// 1. First checks calendar events for explicit work/rest designations
    /// 2. If no explicit designation found, falls back to weekday/weekend logic
    /// 3. For conflicting calendar events, follows priority rules from configuration
    pub async fn is_work_day(&self) -> bool {
        let timestamp = self.dt.and_utc().timestamp_millis();
        let day_type = self.ical.borrow_mut().judge_by_priority(Some(timestamp)).await;
        
        match day_type {
            crate::ical::DayType::NormalDay => {
                // No explicit calendar designation, use weekday logic
                self.is_weekday()
            },
            crate::ical::DayType::WorkDay => {
                // Explicitly marked as work day in calendar
                true
            },
            crate::ical::DayType::RestDay => {
                // Explicitly marked as rest day in calendar
                false
            },
            crate::ical::DayType::ConflictDay => {
                // This should not happen with proper priority rules
                // If it does, default to treating it as a work day
                // Could also panic or log an error based on requirements
                true
            }
        }
    }

    /// Determines if the target date is a weekday based on configuration
    /// 
    /// # Returns
    /// * `true` if the date is a configured workday
    /// * `false` if the date is a weekend day or not configured as workday
    /// 
    /// # Note
    /// * If work_days is None, uses default Saturday and Sunday as weekend
    /// * Weekday numbers: Monday = 1, Tuesday = 2, ..., Sunday = 7
    fn is_weekday(&self) -> bool {
        let weekday_number = self.dt.weekday().number_from_monday() as i8;
        
        if let Some(work_days) = &self.work_days {
            // Use configured work days
            work_days.contains(&weekday_number)
        } else {
            // Default: Monday-Friday are workdays, Saturday-Sunday are weekends
            !matches!(self.dt.weekday(), 
                chrono::Weekday::Sat | chrono::Weekday::Sun
            )
        }
    }
}