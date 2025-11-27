//! Man page display functionality for rexonator

use std::env;
use std::process::Command;

use crate::cli::{Result, TransformError};

/// Display the man page by looking for it relative to the executable
pub fn show_man_page() -> Result<()> {
    // Get the executable path
    let exe_path = env::current_exe()?;
    let exe_dir = exe_path.parent().ok_or_else(|| {
        TransformError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine executable directory",
        ))
    })?;

    // Look for the man page in the same directory as the executable
    let man_page = exe_dir.join("rexonator.1");

    if !man_page.exists() {
        eprintln!("Man page not found at: {}", man_page.display());
        eprintln!("Please ensure rexonator.1 is in the same directory as the executable.");
        eprintln!("\nYou can view it with: man {}", man_page.display());
        std::process::exit(1);
    }

    // Use the man command to display it
    let status = Command::new("man").arg(man_page.as_os_str()).status()?;

    if !status.success() {
        eprintln!("Failed to display man page");
        std::process::exit(1);
    }

    Ok(())
}
