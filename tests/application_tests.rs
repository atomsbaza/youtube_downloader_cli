use ytcli::domain::{Downloader, Video};
use ytcli::application::DownloadVideoUseCase;

struct MockDownloader {
    pub called: std::cell::Cell<bool>,
    pub should_fail: bool,
}

impl Downloader for MockDownloader {
    fn download(&self, _video: &Video) -> Result<(), String> {
        self.called.set(true);
        if self.should_fail {
            Err("mock error".to_string())
        } else {
            Ok(())
        }
    }
}

#[test]
fn test_download_video_usecase_calls_downloader() {
    let mock = MockDownloader { called: std::cell::Cell::new(false), should_fail: false };
    let usecase = DownloadVideoUseCase { downloader: &mock };
    let video = Video {
        url: "https://example.com".to_string(),
        output: Some("output".to_string()),
        quality: "best".to_string(),
        audio_only: false,
        file_type: Some("mp4".to_string()),
    };
    let result = usecase.execute(&video);
    assert!(result.is_ok());
    assert!(mock.called.get());
}

#[test]
fn test_download_video_usecase_error_propagation() {
    let mock = MockDownloader { called: std::cell::Cell::new(false), should_fail: true };
    let usecase = DownloadVideoUseCase { downloader: &mock };
    let video = Video {
        url: "https://fail.com".to_string(),
        output: None,
        quality: "worst".to_string(),
        audio_only: false,
        file_type: Some("webm".to_string()),
    };
    let result = usecase.execute(&video);
    assert!(result.is_err());
    assert!(mock.called.get());
}

#[test]
fn test_download_video_usecase_audio_only() {
    let mock = MockDownloader { called: std::cell::Cell::new(false), should_fail: false };
    let usecase = DownloadVideoUseCase { downloader: &mock };
    let video = Video {
        url: "https://audio.com".to_string(),
        output: Some("audiofile".to_string()),
        quality: "best".to_string(),
        audio_only: true,
        file_type: Some("mp3".to_string()),
    };
    let result = usecase.execute(&video);
    assert!(result.is_ok());
    assert!(mock.called.get());
} 