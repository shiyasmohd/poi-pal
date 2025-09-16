use colored::Colorize;
use update_informer::{registry, Check};

pub struct UpdateChecker;

impl UpdateChecker {
    /// Check for available updates and display notification if found
    pub fn check() {
        // Skip if disabled by env
        if std::env::var("POIPAL_NO_UPDATE_CHECK").is_ok() {
            return;
        }

        let informer = update_informer::new(
            registry::Crates,
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
        );

        if let Ok(Some(version)) = informer.check_version() {
            Self::display_update_notification(&version.to_string());
        }
    }

    /// Display update notification
    fn display_update_notification(new_version: &str) {
        println!();
        println!(
            "{} {} {}",
            "🚀".bold(),
            "New version available:".bright_green().bold(),
            new_version.bright_yellow().bold()
        );
        println!(
            "   {} {}",
            "Update with:".bright_cyan(),
            format!("cargo install {}", env!("CARGO_PKG_NAME")).bright_white()
        );
        println!(
            "   {} {}",
            "Disable check:".bright_black(),
            "export POIPAL_NO_UPDATE_CHECK=1".bright_black()
        );
        println!();
    }
}
