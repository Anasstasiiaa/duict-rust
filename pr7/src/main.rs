use clap::Parser;
use std::fs;
use std::io::{self, Read};
use tokio::task;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    max_threads: Option<usize>,

    file: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let threads = args.max_threads.unwrap_or(num_cpus::get());

    println!("Using {} threads", threads);

    let urls = if let Some(file) = args.file {
        fs::read_to_string(file).unwrap()
    } else {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input).unwrap();
        input
    };

    let urls: Vec<&str> = urls.lines().collect();

    let client = reqwest::Client::new();

    let mut handles = vec![];

    for (i, url) in urls.iter().enumerate() {
        let client = client.clone();
        let url = url.to_string();

        let handle = task::spawn(async move {
            match client.get(&url).send().await {
                Ok(resp) => {
                    let text = resp.text().await.unwrap();
                    let filename = format!("page_{}.html", i);
                    fs::write(&filename, text).unwrap();
                    println!("Saved {}", filename);
                }
                Err(e) => println!("Error: {}", e),
            }
        });

        handles.push(handle);
    }

    for h in handles {
        h.await.unwrap();
    }
}