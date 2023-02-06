mod common;
mod tiktok;

use clap::Parser;
use futures::stream::StreamExt;

#[derive(Parser)]
#[clap(arg_required_else_help(true))]
struct Args {
    /// Users' livestream to be recorded
    #[arg(short, long, required = true)]
    users: Vec<String>,

    /// Folder where user livestreams will be stored
    #[arg(short, long)]
    folder: String,

    /// The account cookie used for sending requests to TikTok
    #[arg(short, long, env)]
    tiktok_cookie: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let folder = args.folder.as_str();
    let cookie = match args.tiktok_cookie.as_ref() {
        Some(cookie) => cookie,
        None => option_env!("TIKTOK_COOKIE")
            .ok_or("Error: Target was not configured with TIKTOK_COOKIE fallback")?,
    };

    let profiles: Vec<crate::tiktok::Profile> = futures::stream::iter(&args.users)
        .filter_map(|username| async move {
            crate::tiktok::Profile::new(username)
                .await
                .map_err(|e| eprintln!("{username} reported: {e}, not downloading"))
                .ok()
        })
        .collect()
        .await;
    dbg!(
        profiles
            .iter()
            .find(|p| p.alive)
            .unwrap()
            .stream_url(cookie)
            .await?
    );

    Ok(())
}
