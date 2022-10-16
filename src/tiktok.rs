pub struct Profile {
    pub username: String,
}

impl Profile {
    pub async fn live_status(&self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let live_page_url = format!("https://www.tiktok.com/@{}/live", self.username);
        let html = crate::common::CLIENT
            .get(&live_page_url)
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
            Ok(Some(id.to_owned()))
        } else {
            Ok(None)
        }
    }

    pub async fn get_stream_url(
        room_id: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let json: serde_json::Value = crate::common::CLIENT
            .post("https://webcast.us.tiktok.com/webcast/room/enter/?aid=1988")
            .form(&[("room_id", room_id)])
            .header(reqwest::header::COOKIE, crate::common::COOKIE)
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

        if let Some(url) = json["data"]["stream_url"]["hls_pull_url"].as_str() {
            return Ok(Some(url.to_owned()));
        }
        Ok(None)
    }
}
