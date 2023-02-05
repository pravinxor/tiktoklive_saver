lazy_static::lazy_static! {
    pub static ref AGENT: ureq::Agent = ureq::AgentBuilder::new().user_agent(USER_AGENT).build();
    pub static ref BARS: indicatif::MultiProgress = indicatif::MultiProgress::new();
}

pub const USER_AGENT: &str = "*/*";
const BUF_SIZE: usize = 32 * 12500; // 3200kb

pub fn download(
    path: impl AsRef<std::path::Path>,
    url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Read;
    use std::io::Write;
    let file = std::fs::File::create(path)?;
    let stream = crate::common::AGENT.get(url).call()?.into_reader();
    let mut writer = std::io::BufWriter::with_capacity(BUF_SIZE, file);
    stream
        .bytes()
        .flatten()
        .for_each(|byte| writer.write_all(&[byte]).unwrap());
    writer.flush()?;
    Ok(())
}
