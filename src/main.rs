use async_std::task;
use bytes::Bytes;
use clap::Parser;
use hostname;
use log::info;
use reqwest;
use serde_json;
use std::{collections::HashMap, time::Duration};

const SAVE_TO: &str = "/var/lib/deployerd/repo";

#[derive(Parser)]
struct Cli {
    gh_user: String,
    gh_repo: String,
}

async fn download_repo(
    client: &reqwest::Client,
    user: &str,
    repo: &str,
) -> Result<Bytes, Box<dyn std::error::Error>> {
    info!("Downloading {}/{}", user, repo);
    // Download the repository

    let url = format!("https://api.github.com/repos/{}/{}/tarball", user, repo);
    let response = client.get(url).send().await?;
    let content = response.bytes().await?;

    println!("Downloaded {} bytes", content.len());
    Ok(content)
}

async fn get_repo_sha(
    client: &reqwest::Client,
    user: &str,
    repo: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Get the latest commit SHA
    let ref_name = "refs/heads/main";

    let url = format!("https://api.github.com/repos/{}/{}/git/refs", user, repo);
    let response = client.get(url).send().await?;
    let content = response.json::<Vec<serde_json::Value>>().await?;

    let commit_sha = content
        .iter().find(|x| x["ref"] == ref_name)
        .unwrap()["object"]["sha"]
        .as_str()
        .unwrap();

    Ok(commit_sha.to_string())
}

fn unzip_repo(content: Bytes) -> Result<(), Box<dyn std::error::Error>> {
    // Unzip the repository

    Ok(())
}

fn make_user_agent() -> String {
    // Get hostname
    let hostname = hostname::get().unwrap().into_string().unwrap();
    format!("deployerd/1.0 ({})", hostname)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let user_agent = make_user_agent();

    // Generate reqwest client
    let client = reqwest::Client::builder().user_agent(user_agent).build()?;

    println!(
        "Starting download loop for {}/{}",
        args.gh_user, args.gh_repo
    );
    loop {
        let commit_sha = get_repo_sha(&client, &args.gh_user, &args.gh_repo).await?;
        println!("Commit SHA: {}", commit_sha);
        // wait 60 seconds
        task::sleep(Duration::from_secs(60)).await;
    }
    Ok(())
}
