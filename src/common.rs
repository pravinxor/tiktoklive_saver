lazy_static::lazy_static! {
    pub static ref CLIENT: reqwest::Client = reqwest::Client::new();
    pub static ref BARS: indicatif::MultiProgress = indicatif::MultiProgress::new();
}

pub const USER_AGENT: &str = "*/*";

use colored::Colorize;
use futures::stream::StreamExt;
use tokio::io::AsyncWriteExt;

pub async fn download_into<U, P>(url: U, location: P) -> Result<(), Box<dyn std::error::Error>>
where
    U: reqwest::IntoUrl,
    P: AsRef<std::path::Path> + std::fmt::Display,
{
    let mut file = tokio::fs::File::create(&location).await?;
    let mut stream = crate::common::CLIENT.get(url).send().await?.bytes_stream();

    let bar = indicatif::ProgressBar::new_spinner();
    let bar = BARS.add(bar);
    bar.set_message(format!("Downloading to {}", &location));
    bar.set_style(indicatif::ProgressStyle::with_template(
        "{msg} [{elapsed}] {spinner}",
    )?);
    while let Some(chunk) = stream.next().await {
        let bytes = chunk?;
        file.write_all(&bytes).await?;
        bar.tick();
    }
    bar.finish_with_message(format!("Completed: {}", location.to_string().green()));
    Ok(())
}
