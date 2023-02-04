lazy_static::lazy_static! {
    pub static ref AGENT: ureq::Agent = ureq::AgentBuilder::new().user_agent(USER_AGENT).build();
    pub static ref BARS: indicatif::MultiProgress = indicatif::MultiProgress::new();
}

pub const USER_AGENT: &str = "*/*";
