//! Work schedule prediction and holiday determination system.
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

use choliday::Choliday;
use clap::Parser;

mod cli;
mod conf;
mod ical;
mod choliday;

/// Main entry point for the work schedule prediction tool
/// 
/// # Usage Examples
/// ```bash
/// # Check if today is a work day using default config
/// choliday -c config.toml
/// 
/// # Check specific date
/// choliday -c config.toml -d 20241225
/// 
/// # Check specific date and time
/// choliday -c config.toml -d 20241225143000
/// ```
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments
    let cli = cli::Cli::parse();
    
    // Create holiday analyzer with configuration
    let choliday = Choliday::new(&cli);
    
    // Determine if target date is a work day
    let is_work_day = choliday.is_work_day().await;
    
    // Output result
    println!("{}", is_work_day);
    
    // Exit with appropriate code for scripting use
    if is_work_day {
        std::process::exit(0);  // Success exit code for work days
    } else {
        std::process::exit(1);  // Non-zero exit code for rest days
    }
}