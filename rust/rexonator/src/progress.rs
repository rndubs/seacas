//! Progress indicator utilities for verbose mode operations
//!
//! This module provides consistent progress bar styling and helper functions
//! for displaying progress during long-running mesh operations.

use indicatif::{ProgressBar, ProgressStyle};

/// Style template for progress bars showing count and percentage
const PROGRESS_TEMPLATE: &str =
    "  {spinner:.green} {msg:<30} [{bar:30.cyan/blue}] {pos}/{len} ({percent}%)";

/// Style template for spinner-only progress (no count)
const SPINNER_TEMPLATE: &str = "  {spinner:.green} {msg}";

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

/// Create a spinner for operations without a known count
pub fn create_spinner(verbose: bool, message: &str) -> Option<ProgressBar> {
    if !verbose {
        return None;
    }

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template(SPINNER_TEMPLATE)
            .expect("Invalid spinner template"),
    );
    pb.set_message(message.to_string());
    Some(pb)
}

/// Increment progress bar if it exists
#[inline]
pub fn inc_progress(pb: &Option<ProgressBar>, delta: u64) {
    if let Some(ref bar) = pb {
        bar.inc(delta);
    }
}

/// Set progress bar position if it exists
#[inline]
pub fn set_progress(pb: &Option<ProgressBar>, pos: u64) {
    if let Some(ref bar) = pb {
        bar.set_position(pos);
    }
}

/// Finish progress bar with a completion message
pub fn finish_progress(pb: Option<ProgressBar>, message: &str) {
    if let Some(bar) = pb {
        bar.finish_with_message(message.to_string());
    }
}

/// Finish progress bar and clear it from display
pub fn finish_and_clear(pb: Option<ProgressBar>) {
    if let Some(bar) = pb {
        bar.finish_and_clear();
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

    #[test]
    fn test_spinner_creation() {
        let spinner = create_spinner(true, "Working...");
        assert!(spinner.is_some());

        let no_spinner = create_spinner(false, "Working...");
        assert!(no_spinner.is_none());
    }
}
