pub struct Profile {
    pub username: String,
}

impl Profile {
    pub async fn live_status(&self) -> Result<Option<u64>, Box<dyn std::error::Error>> {
        let live_page_url = format!("https://www.tiktok.com/@{}/live", self.username);
        let html = crate::common::CLIENT
            .get(&live_page_url)
            .header(reqwest::header::USER_AGENT, crate::common::USER_AGENT)
            .header(reqwest::header::COOKIE, crate::common::COOKIE)
            .send()
            .await?
            .text()
            .await?;

        // Trim the HTML to just the embedded JSON
        let mut json_str;
        let json_open = html
            .find("{\"AppContext")
            .ok_or("Unable to find data json opening")?;
        json_str = &html[json_open..];
        let json_close = json_str
            .find("</script>")
            .ok_or("Unable to find json closing")?;
        json_str = &json_str[..json_close];
        println!("{}", json_str);

        let json = serde_json::from_str(&json_str);
        Ok(None)
    }
}
