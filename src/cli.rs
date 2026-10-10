use std::ffi::OsString;
use std::path::PathBuf;

use clap::Parser;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Auto,
    Mono,
    Neon,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Args {
    pub theme: Theme,
    pub path: PathBuf,
    pub device: Option<String>,
    pub scale: f32,
    pub window: usize,
    pub fps: u32,
    pub loop_playback: bool,
    pub mono: bool,
    pub left_channel: usize,
    pub right_channel: usize,
    pub log: Option<String>,
}

#[derive(Parser, Debug)]
#[command(name = "viboscop")]
#[command(
    version,
    about = "Viboscop stereo XY oscilloscope for WAV and FLAC files"
)]
struct CliArgs {
    #[arg(value_name = "FILE", help = "Audio file to play")]
    path: PathBuf,

    #[arg(long, value_enum, default_value_t = ThemeArg::Auto, help = "Color theme")]
    theme: ThemeArg,

    #[arg(long, help = "Audio output device name")]
    device: Option<String>,

    #[arg(long, default_value_t = 1.0, help = "Initial XY scale")]
    scale: f32,

    #[arg(long, default_value_t = 4096, help = "Visualization sample window")]
    window: usize,

    #[arg(long, default_value_t = 60, help = "Frame rate limit")]
    fps: u32,

    #[arg(long, action = clap::ArgAction::SetTrue, help = "Loop playback")]
    loop_playback: bool,

    #[arg(long, action = clap::ArgAction::SetTrue, help = "Render mono as duplicated stereo")]
    mono: bool,

    #[arg(long, default_value_t = 0, help = "Left channel index")]
    left_channel: usize,

    #[arg(long, default_value_t = 1, help = "Right channel index")]
    right_channel: usize,

    #[arg(long, help = "Logging filter")]
    log: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
enum ThemeArg {
    Auto,
    Mono,
    Neon,
}

impl From<ThemeArg> for Theme {
    fn from(value: ThemeArg) -> Self {
        match value {
            ThemeArg::Auto => Theme::Auto,
            ThemeArg::Mono => Theme::Mono,
            ThemeArg::Neon => Theme::Neon,
        }
    }
}

pub fn parse_args<I, T>(argv: I) -> Result<Args, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = CliArgs::try_parse_from(argv)?;
    Ok(Args {
        theme: cli.theme.into(),
        path: cli.path,
        device: cli.device,
        scale: cli.scale.max(0.001),
        window: cli.window.max(256),
        fps: cli.fps.max(1),
        loop_playback: cli.loop_playback,
        mono: cli.mono,
        left_channel: cli.left_channel,
        right_channel: cli.right_channel,
        log: cli.log,
    })
}
