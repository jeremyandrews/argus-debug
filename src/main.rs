use readability::extractor;
use reqwest;
use std::env;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the URL from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: debug_tool <URL>");
        return Ok(());
    }

    let url = &args[1];

    println!("Loading content from {}", url);

    let res = reqwest::get(url).await?;
    if !res.status().is_success() {
        eprintln!("Error: Status {} - Headers: {:#?}", res.status(), res.headers());
        return Ok(());
    }

    let body = res.text().await?;
    println!("Raw HTML: {}\n-----\n", body);

    let max_retries = 3;
    for retry_count in 0..max_retries {
        let scrape_future = async { extractor::scrape(url) };
        match timeout(Duration::from_secs(60), scrape_future).await {
            Ok(Ok(product)) => {
                println!("Title: {}-----\nBody: {}\n-----", product.title, product.text);
                return Ok(());
            }
            Ok(Err(e)) => {
                eprintln!("Error extracting page: {}", e);
                if retry_count < max_retries - 1 {
                    eprintln!("Retrying... ({}/{})", retry_count + 1, max_retries);
                } else {
                    eprintln!("Failed to extract article after {} retries", retry_count);
                }
            }
            Err(_) => {
                eprintln!("Operation timed out");
                if retry_count < max_retries - 1 {
                    eprintln!("Retrying... ({}/{})", retry_count + 1, max_retries);
                } else {
                    eprintln!("Failed to extract article after {} retries", retry_count);
                }
            }
        }

        if retry_count < max_retries - 1 {
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    eprintln!("Failed to extract content from the provided URL");
    Ok(())
}

