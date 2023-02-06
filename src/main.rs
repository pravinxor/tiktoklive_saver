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

    let mut profiles: Vec<_> = futures::stream::iter(&args.users)
        .filter_map(|username| async move {
            crate::tiktok::Profile::new(username)
                .await
                .map_err(|e| eprintln!("{username} reported: {e}, not downloading"))
                .ok()
        })
        .collect()
        .await;
    let mut active_downloads = std::collections::HashMap::<u64, _>::new();
    loop {
        if let Err(e) = crate::tiktok::Profile::update_alive(&mut profiles).await {
            crate::common::BARS.println(format!("Failed to update live status': {e}",))?
        }

        for mut profile in profiles.iter_mut().filter(|p| p.alive && !p.downloading) {
            let url = match profile.stream_url(cookie).await {
                Ok(url) => url,
                Err(e) => {
                    crate::common::BARS.println(format!(
                        "Failed to get stream URL for {} : {e}",
                        profile.username
                    ))?;
                    continue;
                }
            };
            let time = chrono::offset::Local::now().format("%Y-%m-%d-%H-%M");

            let filename = format!("{folder}{}{time}", profile.username);
            profile.downloading = true;
            active_downloads.insert(profile.room_id, crate::common::download(filename, url));
        }

        for mut profile in profiles.iter_mut().filter(|p| !p.alive && p.downloading) {
            profile.downloading = false;
            if let Err(e) = active_downloads.remove(&profile.room_id).unwrap().await {
                crate::common::BARS
                    .println(format!("{} downloader reported: {e}", profile.username))?;
            }
        }
    }
}
