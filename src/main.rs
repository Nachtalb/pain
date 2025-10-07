use clap::{Parser, ValueEnum};
use futures_util::StreamExt;
use serde_json::Value;
use std::env;
use std::fs::File;
use std::io::prelude::*;

#[derive(ValueEnum, Debug, Clone)]
enum FileType {
    Gif,
    MP4,
    Webp,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// API Key (can be set with GIPHY_API_KEY env var)
    #[arg(short, long)]
    api_key: Option<String>,

    /// Tag to search for
    #[arg(short, long, default_value_t = String::from("pain"))]
    tag: String,

    /// Output File
    #[arg(short, long, default_value_t = String::from("/tmp/{name}.{ext}"))]
    output: String,

    /// Filetype to download
    #[arg(value_enum, short, long, default_value_t = FileType::Webp)]
    filetype: FileType,

    /// Open in webbrowser
    #[arg(short = 'w', long, default_value_t = false)]
    open: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let client = reqwest::Client::new();

    let api_key = match env::var("GIPHY_API_KEY") {
        Ok(key) => Some(key),
        Err(_) => args.api_key,
    }
    .unwrap_or_else(|| {
        println!("No API key provided");
        std::process::exit(1);
    });

    let url = format!(
        "https://api.giphy.com/v1/gifs/random?api_key={}&tag={}",
        api_key, args.tag
    );

    let extension = match args.filetype {
        FileType::Gif => "gif",
        FileType::MP4 => "mp4",
        FileType::Webp => "webp",
    };
    let output_file = args
        .output
        .replacen("{name}", &args.tag, 1)
        .replacen("{ext}", extension, 1);

    let data = client.get(url).send().await?.json::<Value>().await?;

    let type_name = match args.filetype {
        FileType::Gif => "url",
        FileType::MP4 => "mp4",
        FileType::Webp => "webp",
    };
    if let Some(url) = data["data"]["images"]["original"][type_name].as_str() {
        let response = client.get(url).send().await?.error_for_status()?;
        let mut file = File::create(&output_file)?;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk)?;
        }

        println!("{}", output_file);
    };

    if args.open {
        webbrowser::open(&output_file).unwrap();
    }

    Ok(())
}
