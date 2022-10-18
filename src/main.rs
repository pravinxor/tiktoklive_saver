#[path = "tiktok.rs"]
mod tiktok;

#[path = "common.rs"]
mod common;

use clap::Parser;
use colored::Colorize;

#[derive(Parser)]
#[clap(arg_required_else_help(true))]
struct Args {
    /// User's livestream to be recorded
    #[arg(short, long)]
    user: Vec<String>,

    /// Folder where user livestreams will be stored
    #[arg(short, long)]
    folder: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let folder = &args.folder;
    let profiles = args.user.iter().map(|u| crate::tiktok::Profile {
        username: u.to_owned(),
    });
    let cookie =
        std::env::var("TIKTOK_COOKIE").unwrap_or_else(|_| String::from(crate::common::COOKIE));

    tokio_scoped::scope(|s| {
        for profile in profiles {
            let cookie = cookie.as_str();
            s.spawn(async move {
                loop {
                    let url = profile.wait_for_stream_url(cookie).await;
                    let time = chrono::offset::Local::now().format("%Y-%m-%d-%H-%M");
                    if let Err(e) = crate::common::download_into(
                        &url,
                        format!("{}/{}-{}.flv", folder, &profile.username, time),
                    )
                    .await
                    {
                        crate::common::BARS
                            .println(format!(
                                "thread {} reported: {}",
                                &profile.username,
                                e.to_string().red()
                            ))
                            .unwrap();
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
            });
        }
    });
    Ok(())
}
