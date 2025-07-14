pub struct Video {
    pub url: String,
    pub output: Option<String>,
    pub quality: String,
    pub audio_only: bool,
    pub file_type: Option<String>,
}

pub trait Downloader {
    fn download(&self, video: &Video) -> Result<(), String>;
} 