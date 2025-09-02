// main.rs
use feed_rs::parser;
use reqwest;
use rss::{Channel, ChannelBuilder, Item, ItemBuilder};
use std::error::Error;
use std::fs;
use std::collections::{HashMap, HashSet};
use chrono::{DateTime, FixedOffset, Utc};
use tokio;
use serde::Deserialize;
use regex::Regex;

// Config struct for deserializing config.toml
#[derive(Debug, Deserialize)]
struct Config {
    max_items: Option<usize>,
    repo_name: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Read configuration from config.toml
    let config: Config = fs::read_to_string("config.toml")
        .ok()
        .and_then(|contents| toml::from_str(&contents).ok())
        .unwrap_or(Config { max_items: None, repo_name: None });
    let max_items = config.max_items.unwrap_or(300);
    let repo_name = config.repo_name.unwrap_or_else(|| 
        "xavwe/rss-aggregator".to_string()
    );
    println!("Using max_items = {}", max_items);

    // Read feed URLs from "feeds.txt" (one URL per line)
    let feeds_content = fs::read_to_string("feeds.txt")?;
    let feed_urls: Vec<String> = feeds_content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|s| s.to_string())
        .collect();

    if feed_urls.is_empty() {
        eprintln!("No feed URLs found in feeds.txt");
        return Ok(());
    }

    // Concurrently fetch and parse feeds
    let mut all_items = Vec::new();
    let mut feed_data_list = Vec::new();
    let mut handles = Vec::new();
    for url in feed_urls {
        let url_owned = url.to_string();
        let handle = tokio::spawn(async move { fetch_feed_data(url_owned).await });
        handles.push(handle);
    }

    // Collect results from tasks
    for handle in handles {
        match handle.await? {
            Ok(feed_data) => {
                all_items.extend(feed_data.items.clone());
                feed_data_list.push(feed_data);
            },
            Err(e) => eprintln!("Error fetching feed: {}", e),
        }
    }

    // Sort items by publication date (newest first)
    all_items.sort_by(|a, b| b.pub_date.cmp(&a.pub_date));

    // Limit the list to the maximum number of items specified (0 means unlimited)
    if max_items > 0 && all_items.len() > max_items {
        all_items.truncate(max_items);
    }

    // Build a master RSS feed from the aggregated items
    let channel = build_master_feed(&all_items, &repo_name);

    // Write the generated RSS feed to a file
    if let Err(e) = fs::write("feeds/master.xml", channel.to_string()) {
        eprintln!("Error writing master feed: {}", e);
        return Err(e.into());
    }
    println!("Master feed generated with {} items", all_items.len());

    // Clean up old individual feed files
    cleanup_old_feeds(&feed_data_list)?;

    // Generate individual feed files - one unique file per feed URL
    for feed_data in feed_data_list {
        // Apply max_items limit to individual feeds too
        let limited_feed_data = if max_items > 0 && feed_data.items.len() > max_items {
            FeedData {
                title: feed_data.title.clone(),
                url: feed_data.url.clone(),
                items: feed_data.items.into_iter().take(max_items).collect(),
            }
        } else {
            feed_data
        };

        // Generate unique filename based on URL and title to ensure one file per feed
        let unique_filename = generate_unique_filename_for_feed(&limited_feed_data.url, &limited_feed_data.title);
        let filepath = format!("feeds/{}.xml", unique_filename);
        
        let individual_channel = build_individual_feed(&limited_feed_data, &repo_name, &unique_filename);
        
        if let Err(e) = fs::write(&filepath, individual_channel.to_string()) {
            eprintln!("Error writing individual feed {}: {}", filepath, e);
            continue; // Continue with other feeds instead of failing completely
        }
        
        println!("Generated individual feed: {} ({} items)", filepath, limited_feed_data.items.len());
    }

    Ok(())
}

// A simple struct to hold the feed item data
#[derive(Debug, Clone)]
struct FeedItem {
    title: String,
    link: String,
    description: Option<String>,
    pub_date: DateTime<FixedOffset>,
}

// Struct to hold both feed metadata and items
#[derive(Debug)]
struct FeedData {
    title: String,
    url: String,
    items: Vec<FeedItem>,
}

/// Fetches a feed from the given URL and parses its items and metadata.
async fn fetch_feed_data(url: String) -> Result<FeedData, Box<dyn Error + Send + Sync>> {
    let response = reqwest::get(&url).await?;
    let bytes = response.bytes().await?;
    let feed = parser::parse(bytes.as_ref())?;

    // Extract feed title
    let feed_title = feed.title
        .map(|t| t.content)
        .unwrap_or_else(|| url.clone());

    // Create a FixedOffset with zero offset.
    let offset = FixedOffset::east_opt(0).unwrap();

    let mut items = Vec::new();
    for entry in feed.entries {
        // Convert published/updated dates to DateTime<FixedOffset>
        let pub_date = entry
            .published
            .map(|d| d.with_timezone(&offset))
            .or(entry.updated.map(|d| d.with_timezone(&offset)))
            .unwrap_or_else(|| Utc::now().with_timezone::<FixedOffset>(&offset));

        // Use the first available link (if any)
        let link = if !entry.links.is_empty() {
            entry.links[0].href.clone()
        } else {
            String::new()
        };

        let title = entry
            .title
            .map(|t| t.content)
            .unwrap_or_else(|| String::from("No title"));

        let description = entry.summary.map(|s| s.content);

        items.push(FeedItem {
            title,
            link,
            description,
            pub_date,
        });
    }

    Ok(FeedData {
        title: feed_title,
        url,
        items,
    })
}

/// Builds an RSS channel (master feed) from the provided items.
fn build_master_feed(items: &[FeedItem], repo_name: &str) -> Channel {
    let rss_items: Vec<Item> = items
        .iter()
        .map(|fi| {
            let mut builder = ItemBuilder::default();
            builder.title(fi.title.clone());
            builder.link(fi.link.clone());
            if let Some(desc) = &fi.description {
                builder.description(desc.clone());
            }
            // Format the publication date as RFC 2822 for RSS
            builder.pub_date(fi.pub_date.to_rfc2822());
            builder.build()
        })
        .collect();

    ChannelBuilder::default()
        .title("Master RSS Feed")
        .link(format!("https://raw.githubusercontent.com/{}/refs/heads/main/feeds/master.xml", repo_name))
        .description("Aggregated RSS feed")
        .items(rss_items)
        .build()
}

/// Builds an RSS channel for an individual feed.
fn build_individual_feed(feed_data: &FeedData, repo_name: &str, filename: &str) -> Channel {
    let rss_items: Vec<Item> = feed_data.items
        .iter()
        .map(|fi| {
            let mut builder = ItemBuilder::default();
            builder.title(fi.title.clone());
            builder.link(fi.link.clone());
            if let Some(desc) = &fi.description {
                builder.description(desc.clone());
            }
            // Format the publication date as RFC 2822 for RSS
            builder.pub_date(fi.pub_date.to_rfc2822());
            builder.build()
        })
        .collect();

    let github_link = format!("https://raw.githubusercontent.com/{}/refs/heads/main/feeds/{}.xml", repo_name, filename);

    ChannelBuilder::default()
        .title(feed_data.title.clone())
        .link(github_link)
        .description(format!("Archived feed from {}", feed_data.url))
        .items(rss_items)
        .build()
}

/// Converts a string to kebab-case for use as a filename.
fn to_kebab_case(input: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9]+").unwrap();
    re.replace_all(input.to_lowercase().as_str(), "-")
        .trim_matches('-')
        .to_string()
}

/// Generates a unique filename for a feed based on URL and title.
/// This ensures one file per feed URL, preventing collisions.
fn generate_unique_filename_for_feed(url: &str, title: &str) -> String {
    // Use a combination of title and URL hash to create unique filenames
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    // Create a hash of the URL to ensure uniqueness
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    let url_hash = hasher.finish();
    
    // Use title as base, but add URL hash for uniqueness
    let base_title = to_kebab_case(title);
    
    // If title is too generic or empty, use domain from URL
    let filename_base = if base_title.is_empty() || base_title.len() < 3 {
        extract_domain_from_url(url).unwrap_or_else(|| "feed".to_string())
    } else {
        base_title
    };
    
    // Combine title with short hash of URL for uniqueness
    format!("{}-{:08x}", filename_base, (url_hash & 0xFFFFFFFF) as u32)
}

/// Extracts domain name from URL for use in filename.
fn extract_domain_from_url(url: &str) -> Option<String> {
    if let Some(start) = url.find("://") {
        let after_scheme = &url[start + 3..];
        if let Some(end) = after_scheme.find('/') {
            let domain = &after_scheme[..end];
            // Remove www. prefix and convert to kebab case
            let clean_domain = domain.strip_prefix("www.").unwrap_or(domain);
            Some(to_kebab_case(clean_domain))
        } else {
            let clean_domain = after_scheme.strip_prefix("www.").unwrap_or(after_scheme);
            Some(to_kebab_case(clean_domain))
        }
    } else {
        None
    }
}

/// Cleans up old individual feed files that are no longer in the feed list.
fn cleanup_old_feeds(current_feeds: &[FeedData]) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Read current feeds directory
    let feeds_dir = std::path::Path::new("feeds");
    if !feeds_dir.exists() {
        return Ok(());
    }

    // Get current feed URLs as filenames
    let mut current_filenames = HashSet::new();
    
    for feed_data in current_feeds {
        let unique_filename = generate_unique_filename_for_feed(&feed_data.url, &feed_data.title);
        current_filenames.insert(format!("{}.xml", unique_filename));
    }
    
    // Always preserve master.xml
    current_filenames.insert("master.xml".to_string());

    // Read directory and remove files not in current feeds
    for entry in fs::read_dir(feeds_dir)? {
        let entry = entry?;
        let filename = entry.file_name().to_string_lossy().to_string();
        
        // Only remove XML files that aren't in our current set
        if filename.ends_with(".xml") && !current_filenames.contains(&filename) {
            if let Err(e) = fs::remove_file(entry.path()) {
                eprintln!("Warning: Could not remove old feed file {}: {}", filename, e);
            } else {
                println!("Removed old feed file: {}", filename);
            }
        }
    }

    Ok(())
}
