use clap::{Parser, ValueEnum};
use ytcli::application::DownloadVideoUseCase;
use ytcli::domain::Video;
use ytcli::infrastructure::YtDlpDownloader;

/// Supported file types for download
#[derive(ValueEnum, Clone, Debug)]
pub enum FileType {
    Mp4,
    Webm,
    Mp3,
    M4a,
    Wav,
}

impl FileType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileType::Mp4 => "mp4",
            FileType::Webm => "webm",
            FileType::Mp3 => "mp3",
            FileType::M4a => "m4a",
            FileType::Wav => "wav",
        }
    }
}

/// Download YouTube videos via yt-dlp
#[derive(Parser, Debug)]
#[command(name = "ytcli", version = "0.1", author = "คุณ")]
#[command(about = "A simple CLI YouTube downloader using yt-dlp")]
struct Cli {
    /// YouTube video URL or playlist URL
    url: String,

    /// Output filename (without extension)
    #[arg(short, long)]
    output: Option<String>,

    /// Download audio only
    #[arg(short = 'a', long)]
    audio_only: bool,

    /// Video quality (best, worst, 720p, 480p, 360p, 240p, 144p)
    #[arg(short = 'q', long, default_value = "bv*+ba/b")]
    quality: String,

    /// File type/extension to download (mp4, webm, mp3, m4a, wav)
    #[arg(short = 't', long, value_enum)]
    file_type: Option<FileType>,

    /// Playlist: download only from this index (1-based)
    #[arg(long)]
    playlist_start: Option<usize>,

    /// Playlist: download up to this index (inclusive, 1-based)
    #[arg(long)]
    playlist_end: Option<usize>,

    /// Playlist: comma-separated indices or ranges (e.g. 1,3,5-7)
    #[arg(long)]
    playlist_items: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    let video = Video {
        url: cli.url.clone(),
        output: cli.output.clone(),
        save_dir: None,
        quality: cli.quality.clone(),
        audio_only: cli.audio_only,
        file_type: cli.file_type.map(|ft| ft.as_str().to_string()),
        playlist_start: cli.playlist_start,
        playlist_end: cli.playlist_end,
        playlist_items: cli.playlist_items,
    };
    let downloader = YtDlpDownloader;
    let usecase = DownloadVideoUseCase { downloader: &downloader };
    println!("🎬 Starting download: {}", video.url);
    match usecase.execute(&video) {
        Ok(()) => {
            println!("✅ Download complete!");
            if video.audio_only {
                println!("🎵 Audio extracted to {} format", video.file_type.as_deref().unwrap_or("mp3"));
            }
        },
        Err(e) => {
            eprintln!("❌ Download failed: {}", e);
            eprintln!("💡 Make sure yt-dlp is installed: pip install yt-dlp");
        }
    }
}
