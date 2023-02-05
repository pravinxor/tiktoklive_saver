#[derive(Debug, Clone)]
pub struct Profile {
    pub username: String,
    pub room_id: u64,
    pub alive: bool,
    pub downloading: bool,
}

impl Profile {
    fn get_room_id(
        username: impl AsRef<str> + std::fmt::Display,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        let live_page_url = format!("https://www.tiktok.com/@{username}/live");
        let response = crate::common::AGENT.get(&live_page_url).call()?;
        let html = response.into_string()?;

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
        Ok(room_id
            .as_str()
            .ok_or("room_id is not a string, user may not exist or cannot go live")?
            .parse()?)
    }

    pub fn update_alive<'a>(profiles: &mut [&mut Self]) -> Result<(), Box<dyn std::error::Error>> {
        //let mut profiles: Vec<&mut Self> = profiles.into_iter().collect();
        let ids = profiles
            .iter()
            //.filter(|p| predicate(p))
            .map(|profile| profile.room_id.to_string())
            .reduce(|f, s| f + "," + &s)
            .unwrap();
        let response = crate::common::AGENT
            .post("https://webcast.us.tiktok.com/webcast/room/check_alive/?aid=1988")
            .send_form(&[("room_ids", &ids)])?;
        if response.status() % 100 == 4 {
            return Err(response.status_text().into());
        }

        let json: serde_json::Value = response.into_json()?;
        let data = json["data"].as_array().ok_or("data is not an array")?;
        let alive_rooms: std::collections::HashSet<u64> = data
            .iter()
            .filter(|status| status["alive"].as_bool() == Some(true))
            .flat_map(|status| status["room_id"].as_u64())
            .collect();
        profiles
            .iter_mut()
            //.filter(|p| predicate(p))
            .filter(|profile| alive_rooms.contains(&profile.room_id))
            .for_each(|profile| profile.alive = true);
        Ok(())
    }

    pub fn stream_url(&self, cookie: &str) -> Result<String, Box<dyn std::error::Error>> {
        if !self.alive {
            return Err("Stream must be alive to download".into());
        }

        let response = crate::common::AGENT
            .post("https://webcast.us.tiktok.com/webcast/room/enter/?aid=1988")
            .set("cookie", cookie)
            .send_form(&[("room_id", self.room_id.to_string().as_str())])?;
        if response.status() % 100 == 4 {
            return Err(response.status_text().into());
        }

        let json: serde_json::Value = response.into_json()?;
        if let Some(message) = json["data"]["message"].as_str() {
            return Err(message.into());
        }

        if let Some(url) = json["data"]["stream_url"]["rtmp_pull_url"].as_str() {
            Ok(url.to_owned())
        } else {
            Err("rtmp_pull URL missing".into())
        }
    }

    pub fn new(username: String) -> Result<Self, Box<dyn std::error::Error>> {
        let room_id = Self::get_room_id(&username)?;
        Ok(Self {
            username,
            room_id,
            alive: false,
            downloading: false,
        })
    }
}
