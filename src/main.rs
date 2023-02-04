#[path = "tiktok.rs"]
mod tiktok;

#[path = "common.rs"]
mod common;

use clap::Parser;

#[derive(Parser)]
#[clap(arg_required_else_help(true))]
struct Args {
    /// User's livestream to be recorded
    #[arg(short, long, required = true)]
    user: Vec<String>,

    /// Folder where user livestreams will be stored
    #[arg(short, long)]
    folder: String,

    /// The account cookie used for sending requests to TikTok
    #[arg(short, long, env)]
    tiktok_cookie: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let folder = args.folder;
    let cookie = match args.tiktok_cookie.as_ref() {
        Some(cookie) => cookie,
        None => option_env!("TIKTOK_COOKIE")
            .ok_or("Error: Target was not configured with TIKTOK_COOKIE fallback")?,
    };
    let profiles: Vec<crate::tiktok::Profile> = args
        .user
        .iter()
        .filter_map(|username| {
            crate::tiktok::Profile::new(username.into())
                .map_err(|e| eprintln!("{username} reported: {e}, not downloading"))
                .ok()
        })
        .collect();
    dbg!(profiles);
    Ok(())
}
