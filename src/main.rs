use async_std::task;
use bytes::Bytes;
use clap::Parser;
use env_logger;
use hostname;
use log::info;
use serde_json;
use std::{future::Future, time::Duration};

const SAVE_TO: &str = "~/.local/share/deployerd";
const WAIT_TIME_SECS: u64 = 60;

#[derive(Parser, Debug)]
struct Cli {
    gh_user: String,
    gh_repo: String,
}

async fn get_repo(
    client: &reqwest::Client,
    user: &str,
    repo: &str,
) -> Result<Bytes, Box<dyn std::error::Error>> {
    info!("Downloading {}/{}", user, repo);
    // Download the repository

    let url = format!("https://api.github.com/repos/{}/{}/tarball", user, repo);
    let response = client.get(url).send().await?;
    let content = response.bytes().await?;
    Ok(content)
}

fn unzip_tarball(content: Bytes) -> Result<(), Box<dyn std::error::Error>> {
    info!("Unzipping tarball");
    // Unzip the tarball
    let mut archive =
        tar::Archive::new(flate2::read::GzDecoder::new(std::io::Cursor::new(content)));
    archive.unpack(SAVE_TO)?;
    Ok(())
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
        .iter()
        .find(|x| x["ref"] == ref_name)
        .unwrap()["object"]["sha"]
        .as_str()
        .unwrap();

    Ok(commit_sha.to_string())
}

fn make_user_agent() -> String {
    // Get hostname
    let hostname = hostname::get().unwrap().into_string().unwrap();
    format!("deployerd/1.0 ({})", hostname)
}

async fn process_repo(client: reqwest::Client, user: &str, repo: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = get_repo(&client, &user, &repo).await?;
    unzip_tarball(content)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Cli::parse();

    log::debug!("Starting deployerd");
    log::debug!("Arguments: {:?}", args);

    let user_agent = make_user_agent();

    // Generate reqwest client
    let client = reqwest::Client::builder().user_agent(user_agent).build()?;

    let mut last_known_commit_sha: Option<String> = None;

    log::info!(
        "Starting download loop for {}/{}",
        args.gh_user,
        args.gh_repo
    );
    loop {
        let commit_sha = get_repo_sha(&client, &args.gh_user, &args.gh_repo).await?;
        log::debug!("Commit SHA: {}", commit_sha);

        match last_known_commit_sha {
            Some(ref sha) if sha == &commit_sha => {
                log::debug!("No new commits detected, sleeping");
                continue;
            }
            _ => {
                log::info!("New commit detected, downloading repository");
                match process_repo(client.clone(), &args.gh_user, &args.gh_repo).await {
                    Ok(_) => {
                        log::info!("Repository downloaded successfully");
                        last_known_commit_sha = Some(commit_sha.to_owned());
                    }
                    Err(e) => {
                        log::error!("Error downloading repository: {}", e);
                    }
                }   
            }
        }

        task::sleep(Duration::from_secs(WAIT_TIME_SECS)).await;
    }
}
