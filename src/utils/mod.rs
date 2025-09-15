use colored::Colorize;
use std::collections::BTreeMap;

use crate::models::{Indexer, IndexerPOI, POIGroup};

pub fn display_header(title: &str) {
    println!("\n{}", "=".repeat(100).bright_cyan());
    println!("{}", title.bright_white().bold());
    println!("{}", "=".repeat(100).bright_cyan());
}

pub fn display_subheader(title: &str) {
    println!("\n{}", title.bright_yellow());
    println!("{}", "-".repeat(title.len()).bright_yellow());
}

pub fn display_info(label: &str, value: &str) {
    println!("{}: {}", label.bright_blue(), value.white());
}

pub fn display_success(message: &str) {
    println!("{} {}", "✓".green().bold(), message.green());
}

pub fn display_error(message: &str) {
    println!("{} {}", "✗".red().bold(), message.red());
}

pub fn display_warning(message: &str) {
    println!("{} {}", "⚠".yellow().bold(), message.yellow());
}

pub fn display_progress(message: &str) {
    print!("{} {}... ", "→".bright_cyan(), message);
}

pub fn display_pois(pois: Vec<IndexerPOI>, block: u32, deployment: &str) {
    display_header(&format!(
        "POIs for deployment {} at block {}",
        deployment, block
    ));

    if pois.is_empty() {
        display_warning("No POIs found");
        return;
    }

    // Group POIs by hash
    let mut poi_groups: BTreeMap<String, Vec<IndexerPOI>> = BTreeMap::new();
    for poi in pois {
        poi_groups.entry(poi.poi.clone()).or_default().push(poi);
    }

    let unique_pois = poi_groups.len();
    let total_indexers = poi_groups.values().map(|v| v.len()).sum::<usize>();

    println!(
        "\n{}",
        format!(
            "Found {} indexers with {} unique POI(s)",
            total_indexers, unique_pois
        )
        .bright_green()
    );

    // Display each POI group as a table
    for (poi_hash, indexers) in poi_groups {
        println!("\n{}", "═".repeat(100).bright_cyan());
        println!(
            "{} {}",
            "POI Hash:".bright_yellow().bold(),
            poi_hash.bright_white()
        );
        println!(
            "{} {} indexer(s)",
            "Count:".bright_yellow().bold(),
            indexers.len().to_string().bright_white()
        );
        println!("{}", "─".repeat(100).bright_cyan());

        // Table header
        println!(
            "{:<44} │ {}",
            " Indexer ID".bright_blue().bold(),
            "URL".bright_blue().bold()
        );
        println!("{}", "─".repeat(100).bright_black());

        // Table rows
        for indexer in indexers {
            let truncated_id = if indexer.indexer_id.len() > 42 {
                format!("{}...", &indexer.indexer_id[..39])
            } else {
                indexer.indexer_id.clone()
            };

            let truncated_url = if indexer.indexer_url.len() > 52 {
                format!("{}...", &indexer.indexer_url[..49])
            } else {
                indexer.indexer_url.clone()
            };

            println!(
                " {:<43} │ {}",
                truncated_id.white(),
                truncated_url.bright_black()
            );
        }
    }
    println!("\n{}", "═".repeat(100).bright_cyan());
}

pub fn display_poi_groups(groups: Vec<POIGroup>, block: u32, correct_indexer_id: &str) {
    display_subheader(&format!("POI Groups at block {}", block));

    for group in groups {
        let status_icon = if group.is_correct {
            "✓".green().bold()
        } else {
            "✗".red().bold()
        };

        let status_text = if group.is_correct {
            "CORRECT".green().bold()
        } else {
            "DIVERGED".red().bold()
        };

        println!(
            "\n{} {} POI: {}",
            status_icon,
            status_text,
            group.poi.bright_white()
        );

        println!("  {} ({}):", "Indexers".bright_blue(), group.indexers.len());

        for indexer_id in group.indexers.keys() {
            let marker = if indexer_id == correct_indexer_id {
                " (reference)".bright_magenta().to_string()
            } else {
                String::new()
            };
            println!("    • {}{}", indexer_id.white(), marker);
        }
    }
}

pub fn display_divergence_summary(
    has_divergence: bool,
    diverged_block: Option<u32>,
    start_block: u32,
    end_block: u32,
) {
    println!();
    if has_divergence {
        if let Some(block) = diverged_block {
            display_error(&format!("Divergence found at block {}", block));
        }
    } else {
        display_success(&format!(
            "No divergence found between blocks {} and {}",
            start_block, end_block
        ));
    }
}

pub fn format_deployment_hash(hash: &str) -> String {
    if hash.len() > 16 {
        format!("{}...{}", &hash[..8], &hash[hash.len() - 8..])
    } else {
        hash.to_string()
    }
}

pub fn group_pois_by_hash(
    indexers: &BTreeMap<String, Indexer>,
    pois: &[(String, String)],
    correct_indexer_id: &str,
) -> Vec<POIGroup> {
    let mut poi_groups: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();

    for (indexer_id, poi) in pois {
        let indexer = &indexers[indexer_id];
        poi_groups
            .entry(poi.clone())
            .or_default()
            .insert(indexer_id.clone(), indexer.url.clone());
    }

    poi_groups
        .into_iter()
        .map(|(poi, indexers)| POIGroup {
            is_correct: indexers.contains_key(correct_indexer_id),
            poi,
            indexers,
        })
        .collect()
}
