pub enum SocialPreset {
    InstaPost,  // 1080x1080
    InstaStory, // 1080x1920
    YTThumb,    // 1280x720
}

impl SocialPreset {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "insta-post" => Some(Self::InstaPost),
            "insta-story" => Some(Self::InstaStory),
            "yt" => Some(Self::YTThumb),
            _ => None,
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            Self::InstaPost => (1080, 1080),
            Self::InstaStory => (1080, 1920),
            Self::YTThumb => (1280, 720),
        }
    }
}
