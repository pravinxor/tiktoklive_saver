pub struct Profile {
    pub username: String,
    pub room_id: u64,
    pub alive: bool,
}

impl Profile {
    async fn get_room_id(
        username: impl AsRef<str> + std::fmt::Display,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        let live_page_url = format!("https://www.tiktok.com/@{username}/live");
        let response = crate::common::CLIENT
            .get(&live_page_url)
            .send()
            .await?
            .text()
            .await?;

        // Trim the HTML to just the embedded JSON
        let mut json_str;
        let json_open = response
            .find("{\"AppContext")
            .ok_or("Unable to find data json opening")?;
        json_str = &response[json_open..];
        let json_close = json_str
            .find("</script>")
            .ok_or("Unable to find json closing")?;
        json_str = &json_str[..json_close];

        let json: serde_json::Value = serde_json::from_str(json_str)?;
        // Find the room_id
        let room_id = &json["LiveRoom"]["liveRoomUserInfo"]["user"]["roomId"];
        Ok(room_id
            .as_str()
            .ok_or("room_id is not a string, user may not exist or cannot go live")?
            .parse()?)
    }

    pub async fn update_alive<'a>(profiles: &mut [Self]) -> Result<(), Box<dyn std::error::Error>> {
        if profiles.is_empty() {
            return Ok(());
        }
        let ids = profiles
            .iter()
            .map(|profile| profile.room_id.to_string())
            .reduce(|f, s| f + "," + &s)
            .unwrap();
        let response = crate::common::CLIENT
            .post("https://webcast.us.tiktok.com/webcast/room/check_alive/?aid=1988")
            .form(&[("room_ids", &ids)])
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        let data = json["data"].as_array().ok_or("data is not an array")?;
        let alive_rooms: std::collections::HashSet<u64> = data
            .iter()
            .filter(|status| status["alive"].as_bool() == Some(true))
            .flat_map(|status| status["room_id"].as_u64())
            .collect();
        profiles
            .iter_mut()
            .for_each(|mut profile| profile.alive = alive_rooms.contains(&profile.room_id));
        Ok(())
    }

    pub async fn stream_url(&self, cookie: &str) -> Result<String, Box<dyn std::error::Error>> {
        if !self.alive {
            return Err("Stream must be alive to download".into());
        }

        let response = crate::common::CLIENT
            .post("https://webcast.us.tiktok.com/webcast/room/enter/?aid=1988")
            .header(reqwest::header::COOKIE, cookie)
            .form(&[("room_id", self.room_id.to_string().as_str())])
            .send()
            .await?;
        response.error_for_status_ref()?;

        let json: serde_json::Value = response.json().await?;
        if let Some(message) = json["data"]["message"].as_str() {
            return Err(message.into());
        }

        if let Some(url) = json["data"]["stream_url"]["rtmp_pull_url"].as_str() {
            Ok(url.to_owned())
        } else {
            Err("rtmp_pull URL missing".into())
        }
    }

    pub async fn new(
        username: impl Into<String> + AsRef<str> + std::fmt::Display,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let room_id = Self::get_room_id(&username).await?;
        Ok(Self {
            username: username.into(),
            room_id,
            alive: false,
        })
    }
}
