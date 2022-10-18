use colored::Colorize;

pub struct Profile {
    pub username: String,
}

impl Profile {
    pub async fn room_id(&self, cookie: &str) -> Result<Option<u64>, Box<dyn std::error::Error>> {
        let live_page_url = format!("https://www.tiktok.com/@{}/live", self.username);
        let html = crate::common::CLIENT
            .get(&live_page_url)
            .header(reqwest::header::COOKIE, cookie)
            .header(reqwest::header::USER_AGENT, crate::common::USER_AGENT)
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

        let json: serde_json::Value = serde_json::from_str(json_str)?;

        // Find the room_id
        let room_id = &json["LiveRoom"]["liveRoomUserInfo"]["user"]["roomId"];

        if let Some(id) = room_id.as_str() {
            let id = id.parse()?;
            Ok(Some(id))
        } else {
            Ok(None)
        }
    }

    pub async fn get_stream_url(
        room_id: u64,
        cookie: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let json: serde_json::Value = crate::common::CLIENT
            .post("https://webcast.us.tiktok.com/webcast/room/enter/?aid=1988")
            .form(&[("room_id", room_id)])
            .header(reqwest::header::COOKIE, cookie)
            .header(reqwest::header::USER_AGENT, crate::common::USER_AGENT)
            .send()
            .await?
            .json()
            .await?;

        // Report any error messages
        if let Some(message) = json["data"]["message"].as_str() {
            match message {
                "User doesn't login" => return Err("Missing or invalid cookie".into()),
                "room has finished" => return Ok(None),
                _ => return Err(message.into()),
            }
        }
        if let Some(url) = json["data"]["stream_url"]["rtmp_pull_url"].as_str() {
            return Ok(Some(url.to_owned()));
        }
        Ok(None)
    }

    pub async fn wait_for_stream_url(&self, cookie: &str) -> String {
        let bar = indicatif::ProgressBar::new_spinner();
        let bar = crate::common::BARS.add(bar);
        bar.set_message(format!("Waiting for {}'s live to start", self.username));
        bar.set_style(indicatif::ProgressStyle::with_template("{msg} {spinner}").unwrap());
        bar.enable_steady_tick(std::time::Duration::from_secs(1));
        loop {
            let id = match self.room_id(cookie).await {
                Ok(id) => id,
                Err(e) => {
                    crate::common::BARS
                        .println(format!(
                            "thread {} reported: {}",
                            self.username,
                            e.to_string().red()
                        ))
                        .unwrap();
                    continue;
                }
            };
            if let Some(id) = id {
                match Self::get_stream_url(id, cookie).await {
                    Ok(url) => {
                        if let Some(url) = url {
                            bar.finish_and_clear();
                            return url;
                        }
                    }
                    Err(e) => crate::common::BARS
                        .println(format!(
                            "thread {} reported: {}",
                            self.username,
                            e.to_string().red()
                        ))
                        .unwrap(),
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }
    }
}
