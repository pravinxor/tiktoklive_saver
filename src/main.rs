#[path = "tiktok.rs"]
mod tiktok;

#[path = "common.rs"]
mod common;

use clap::Parser;
use std::ops::DerefMut;

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
    let folder = args.folder.as_str();
    let cookie = match args.tiktok_cookie.as_ref() {
        Some(cookie) => cookie,
        None => option_env!("TIKTOK_COOKIE")
            .ok_or("Error: Target was not configured with TIKTOK_COOKIE fallback")?,
    };
    let mut profiles: Vec<std::sync::Arc<std::sync::Mutex<crate::tiktok::Profile>>> = args
        .user
        .iter()
        .filter_map(|username| {
            crate::tiktok::Profile::new(username.into())
                .map_err(|e| eprintln!("{username} reported: {e}, not downloading"))
                .ok()
        })
        .map(|p| std::sync::Arc::new(std::sync::Mutex::new(p)))
        .collect();
    std::thread::scope(|s| {
        let mut inactive_profiles: Vec<std::sync::MutexGuard<crate::tiktok::Profile>> = profiles
            .iter_mut()
            .flat_map(|m| m.try_lock())
            .filter(|p| !p.alive)
            .collect();
        let inactive_refs = inactive_profiles.iter_mut().map(|p| p.deref_mut());
        crate::tiktok::Profile::update_alive(inactive_refs).unwrap();
        dbg!(inactive_profiles
            .iter()
            .find(|p| p.alive)
            .unwrap()
            .stream_url(cookie)
            .unwrap());
    });
    Ok(())
}
