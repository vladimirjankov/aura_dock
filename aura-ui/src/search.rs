use gtk::prelude::*;
use gtk::SearchEntry;
use std::process::Command;

/// Creates and configures the search bar widget
pub fn create_search_bar() -> SearchEntry {
    let search_entry = SearchEntry::new();
    search_entry.set_placeholder_text(Some("Search Perplexity..."));
    search_entry.set_width_chars(20);
    search_entry.add_css_class("dock-search");
    
    search_entry.connect_activate(|entry| {
        let query = entry.text();
        if !query.is_empty() {
            open_perplexity_search(&query);
            entry.set_text("");
        }
    });

    search_entry
}

/// URL-encodes a string for use in a query parameter
fn url_encode(input: &str) -> String {
    input.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' {
                c.to_string()
            } else if c == ' ' {
                "+".to_string()
            } else {
                format!("%{:02X}", c as u8)
            }
        })
        .collect()
}

/// Opens a Perplexity search in the default browser
fn open_perplexity_search(query: &str) {
    let encoded = url_encode(query);
    let url = format!("https://www.perplexity.ai/search?q={}", encoded);
    
    if let Err(e) = Command::new("xdg-open").arg(&url).spawn() {
        eprintln!("Failed to open browser: {}", e);
    }
}
