lazy_static::lazy_static! {
    pub static ref CLIENT: reqwest::Client = reqwest::ClientBuilder::new().user_agent(USER_AGENT).build().unwrap();
    pub static ref BARS: indicatif::MultiProgress = indicatif::MultiProgress::new();
}

pub const USER_AGENT: &str = "*/*";

use futures::stream::StreamExt;
use tokio::io::AsyncWriteExt;

pub async fn download(
    path: impl AsRef<std::path::Path>,
    url: impl reqwest::IntoUrl,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = tokio::fs::File::create(path).await?;
    let mut stream = crate::common::CLIENT.get(url).send().await?.bytes_stream();
    while let Some(chunk) = stream.next().await {
        file.write_all(&chunk?).await?;
    }
    Ok(())
}
