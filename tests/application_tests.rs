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

fn make_video(url: &str, audio_only: bool, file_type: &str) -> Video {
    Video {
        url: url.to_string(),
        output: None,
        save_dir: None,
        quality: "best".to_string(),
        audio_only,
        file_type: Some(file_type.to_string()),
        playlist_start: None,
        playlist_end: None,
        playlist_items: None,
    }
}

#[test]
fn test_download_video_usecase_calls_downloader() {
    let mock = MockDownloader { called: std::cell::Cell::new(false), should_fail: false };
    let usecase = DownloadVideoUseCase { downloader: &mock };
    let video = Video {
        url: "https://example.com".to_string(),
        output: Some("output".to_string()),
        save_dir: None,
        quality: "best".to_string(),
        audio_only: false,
        file_type: Some("mp4".to_string()),
        playlist_start: None,
        playlist_end: None,
        playlist_items: None,
    };
    let result = usecase.execute(&video);
    assert!(result.is_ok());
    assert!(mock.called.get());
}

#[test]
fn test_download_video_usecase_error_propagation() {
    let mock = MockDownloader { called: std::cell::Cell::new(false), should_fail: true };
    let usecase = DownloadVideoUseCase { downloader: &mock };
    let result = usecase.execute(&make_video("https://fail.com", false, "webm"));
    assert!(result.is_err());
    assert!(mock.called.get());
}

#[test]
fn test_download_video_usecase_audio_only() {
    let mock = MockDownloader { called: std::cell::Cell::new(false), should_fail: false };
    let usecase = DownloadVideoUseCase { downloader: &mock };
    let result = usecase.execute(&make_video("https://audio.com", true, "mp3"));
    assert!(result.is_ok());
    assert!(mock.called.get());
}

#[test]
fn test_download_with_save_dir() {
    let mock = MockDownloader { called: std::cell::Cell::new(false), should_fail: false };
    let usecase = DownloadVideoUseCase { downloader: &mock };
    let video = Video {
        url: "https://example.com".to_string(),
        output: Some("myvideo".to_string()),
        save_dir: Some("/tmp/downloads".to_string()),
        quality: "720p".to_string(),
        audio_only: false,
        file_type: Some("mp4".to_string()),
        playlist_start: None,
        playlist_end: None,
        playlist_items: None,
    };
    let result = usecase.execute(&video);
    assert!(result.is_ok());
    assert!(mock.called.get());
}

#[test]
fn test_download_with_playlist_items() {
    let mock = MockDownloader { called: std::cell::Cell::new(false), should_fail: false };
    let usecase = DownloadVideoUseCase { downloader: &mock };
    let video = Video {
        url: "https://example.com/playlist".to_string(),
        output: None,
        save_dir: None,
        quality: "best".to_string(),
        audio_only: false,
        file_type: Some("mp4".to_string()),
        playlist_start: Some(2),
        playlist_end: Some(10),
        playlist_items: Some("1,3,5-7".to_string()),
    };
    let result = usecase.execute(&video);
    assert!(result.is_ok());
    assert!(mock.called.get());
}
