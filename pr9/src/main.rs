use clap::Parser;
use futures::stream::{self, StreamExt};
use std::fs;
use std::io::{self, Read};

#[derive(Parser)]
struct Args {
    #[arg(long)]
    max_threads: Option<usize>,

    file: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let concurrency = args.max_threads.unwrap_or(num_cpus::get());

    println!("Max concurrency: {}", concurrency);

    let input = if let Some(file) = args.file {
        fs::read_to_string(file).unwrap()
    } else {
        let mut s = String::new();
        io::stdin().read_to_string(&mut s).unwrap();
        s
    };

    let urls: Vec<String> = input.lines().map(|s| s.to_string()).collect();

    let client = reqwest::Client::new();

    stream::iter(urls.into_iter().enumerate())
        .map(|(i, url)| {
            let client = client.clone();

            async move {
                match client.get(&url).send().await {
                    Ok(resp) => {
                        match resp.text().await {
                            Ok(text) => {
                                let filename = format!("page_{}.html", i);
                                fs::write(&filename, text).unwrap();
                                println!("Saved {}", filename);
                            }
                            Err(e) => println!("Error reading body: {}", e),
                        }
                    }
                    Err(e) => println!("Request error: {}", e),
                }
            }
        })
        .buffer_unordered(concurrency)
        .collect::<Vec<_>>()
        .await;
}