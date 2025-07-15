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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_creation() {
        let video = Video {
            url: "https://example.com".to_string(),
            output: Some("output".to_string()),
            quality: "best".to_string(),
            audio_only: false,
            file_type: Some("mp4".to_string()),
        };
        assert_eq!(video.url, "https://example.com");
        assert_eq!(video.output.as_deref(), Some("output"));
        assert_eq!(video.quality, "best");
        assert!(!video.audio_only);
        assert_eq!(video.file_type.as_deref(), Some("mp4"));
    }

    #[test]
    fn test_video_audio_only() {
        let video = Video {
            url: "https://audio.com".to_string(),
            output: None,
            quality: "worst".to_string(),
            audio_only: true,
            file_type: Some("mp3".to_string()),
        };
        assert!(video.audio_only);
        assert_eq!(video.file_type.as_deref(), Some("mp3"));
    }
} 