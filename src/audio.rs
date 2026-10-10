use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::get_codecs;
use symphonia::default::get_probe;

#[derive(Clone, Debug)]
pub struct AudioFile {
    pub path: String,
    pub sample_rate: u32,
    pub channels: usize,
    pub samples: Vec<f32>,
}

pub fn dup_mono_to_stereo(mono: &[f32]) -> Vec<f32> {
    let mut out = Vec::with_capacity(mono.len() * 2);
    for &sample in mono {
        out.push(sample);
        out.push(sample);
    }
    out
}

pub fn select_pair(samples: &[f32], left_idx: usize, right_idx: usize) -> Vec<(f32, f32)> {
    let mut out = Vec::new();
    let stride = 2usize;
    for chunk in samples.chunks(stride) {
        if chunk.len() < stride {
            break;
        }
        let left = chunk
            .get(left_idx.min(chunk.len() - 1))
            .copied()
            .unwrap_or(0.0);
        let right = chunk
            .get(right_idx.min(chunk.len() - 1))
            .copied()
            .unwrap_or(0.0);
        out.push((left, right));
    }
    out
}

pub fn load_audio_file(path: &Path) -> Result<AudioFile> {
    let file = File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut probed = get_probe()
        .format(
            &Hint::new(),
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .with_context(|| format!("failed to probe audio file: {}", path.display()))?;

    let track = probed
        .format
        .tracks()
        .iter()
        .find(|track| track.codec_params.codec != CODEC_TYPE_NULL)
        .context("no playable audio track found")?
        .clone();

    let mut decoder = get_codecs().make(&track.codec_params, &DecoderOptions::default())?;
    let mut samples = Vec::new();
    let mut sample_rate = 44100u32;
    let mut channels = 2usize;

    loop {
        match probed.format.next_packet() {
            Ok(packet) => {
                let decoded = match decoder.decode(&packet) {
                    Ok(decoded) => decoded,
                    Err(err) => {
                        if matches!(err, SymError::DecodeError(_)) {
                            continue;
                        }
                        return Err(err.into());
                    }
                };

                let spec = decoded.spec();
                sample_rate = spec.rate;
                channels = spec.channels.count();
                let mut sample_buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, *spec);
                sample_buf.copy_interleaved_ref(decoded);
                samples.extend_from_slice(sample_buf.samples());
            }
            Err(err) => match err {
                SymError::IoError(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => break,
                SymError::DecodeError(_) => continue,
                _ => return Err(err.into()),
            },
        }
    }

    if samples.is_empty() {
        return Err(anyhow::anyhow!(
            "no audio samples were decoded from {}",
            path.display()
        ));
    }

    let mut interleaved = if channels == 1 {
        dup_mono_to_stereo(&samples)
    } else if channels > 2 {
        let mut downmixed = Vec::new();
        let total = samples.len();
        let frames = total / channels;
        for frame in 0..frames {
            let base = frame * channels;
            let left = samples.get(base).copied().unwrap_or(0.0);
            let right = samples
                .get(base + 1.min(channels - 1))
                .copied()
                .unwrap_or(0.0);
            downmixed.push(left);
            downmixed.push(right);
        }
        downmixed
    } else {
        samples
    };

    if interleaved.len() % 2 != 0 {
        interleaved.pop();
    }

    Ok(AudioFile {
        path: path.to_string_lossy().to_string(),
        sample_rate,
        channels: 2,
        samples: interleaved,
    })
}
