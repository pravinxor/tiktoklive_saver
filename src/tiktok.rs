#[derive(Debug)]
pub struct Profile {
    pub username: String,
    pub room_id: u64,
    pub downloading: std::sync::atomic::AtomicBool,
}

impl Profile {
    fn get_room_id<S>(username: S) -> Result<u64, Box<dyn std::error::Error>>
    where
        S: AsRef<str> + std::fmt::Display,
    {
        let live_page_url = format!("https://www.tiktok.com/@{username}/live");
        let response = crate::common::AGENT
            .get(&live_page_url)
            .set("User-Agent", crate::common::USER_AGENT)
            .call()?;
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

    pub fn new(username: String) -> Result<Self, Box<dyn std::error::Error>> {
        let room_id = Self::get_room_id(&username)?;
        Ok(Self {
            username,
            room_id,
            downloading: false.into(),
        })
    }
}
