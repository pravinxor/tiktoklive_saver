#[path = "tiktok.rs"]
mod tiktok;

#[path = "common.rs"]
mod common;

use clap::Parser;

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

    tokio_scoped::scope(|s| {
        for profile in profiles {
            s.spawn(async move {
                loop {
                    let url = profile.wait_for_stream_url().await;
                    eprintln!("Opening livestream for {}", &profile.username);
                    if let Err(e) = crate::common::download_into(
                        &url,
                        format!("{}/{}.flv", folder, &profile.username),
                    )
                    .await
                    {
                        eprintln!("{}", e);
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            });
        }
    });
    Ok(())
}
