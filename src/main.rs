//! Main entry point for ccusage-rs
//! 
//! This is the main binary entry point for the Claude Code usage analysis tool.

use ccusage_rs::{cli::App, error::Result, logging};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let app = App::new();
    
    // Initialize logging
    logging::init_logging(app.verbose)?;
    
    // Run the application
    app.run().await
}
