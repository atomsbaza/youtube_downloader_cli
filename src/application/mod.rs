use crate::domain::{Downloader, Video};

pub struct DownloadVideoUseCase<'a> {
    pub downloader: &'a dyn Downloader,
}

impl<'a> DownloadVideoUseCase<'a> {
    pub fn execute(&self, video: &Video) -> Result<(), String> {
        self.downloader.download(video)
    }
} 