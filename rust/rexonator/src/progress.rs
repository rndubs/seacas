//! Progress indicator utilities for verbose mode operations
//!
//! This module provides consistent progress bar styling and helper functions
//! for displaying progress during long-running mesh operations.

use indicatif::{ProgressBar, ProgressStyle};

/// Style template for progress bars showing count and percentage
const PROGRESS_TEMPLATE: &str =
    "  {spinner:.green} {msg:<30} [{bar:30.cyan/blue}] {pos}/{len} ({percent}%)";

/// Create a progress bar for operations with a known count
///
/// Returns `None` if verbose mode is disabled, allowing callers to skip
/// progress updates when not needed.
pub fn create_progress_bar(verbose: bool, total: u64, message: &str) -> Option<ProgressBar> {
    if !verbose || total == 0 {
        return None;
    }

    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(PROGRESS_TEMPLATE)
            .expect("Invalid progress template")
            .progress_chars("=>-"),
    );
    pb.set_message(message.to_string());
    Some(pb)
}

/// Finish progress bar, keeping it visible on screen
pub fn finish_progress(pb: Option<ProgressBar>) {
    if let Some(bar) = pb {
        bar.finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_creation_verbose() {
        let pb = create_progress_bar(true, 100, "Test progress");
        assert!(pb.is_some());
    }

    #[test]
    fn test_progress_bar_creation_not_verbose() {
        let pb = create_progress_bar(false, 100, "Test progress");
        assert!(pb.is_none());
    }

    #[test]
    fn test_progress_bar_zero_total() {
        let pb = create_progress_bar(true, 0, "Empty progress");
        assert!(pb.is_none());
    }
}
