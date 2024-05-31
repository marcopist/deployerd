use clap::Parser;

#[derive(Parser)]
struct Cli {
    gh_user: String,
    gh_repo: String,
}

fn main() {
    let args = Cli::parse();
    println!("User: {}", args.gh_user);
    println!("Repo: {}", args.gh_repo);
}