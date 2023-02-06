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
    let bar = crate::common::BARS.add(indicatif::ProgressBar::new_spinner());
    bar.set_style(indicatif::ProgressStyle::with_template("{msg} {spinner}")?);
    loop {
        bar.set_message("Checking for active streams");
        bar.tick();
        if let Err(e) = crate::tiktok::Profile::update_alive(&mut profiles).await {
            crate::common::BARS.println(format!("Failed to update live status': {e}",))?
        }

        for profile in &profiles {
            if !profile.alive || active_downloads.contains_key(&profile.room_id) {
                continue;
            }
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
            let handle = tokio::spawn(crate::common::download(filename, url));

            active_downloads.insert(profile.room_id, handle);
        }

        for (room_id, handle) in &mut active_downloads {
            if handle.is_finished() {
                if let Err(e) = handle.await? {
                    crate::common::BARS
                        .println(format!("Download thread for room: {room_id} reported {e}"))?;
                }
            }
        }
    }
}
