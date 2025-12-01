use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use std::sync::{Arc, Mutex};

pub struct AudioOutput {
    _stream: Stream,
}

impl AudioOutput {
    pub fn new(audio_buffer: Arc<Mutex<Vec<f32>>>) -> Result<Self, String> {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .ok_or("No output device available")?;

        let config = device
            .default_output_config()
            .map_err(|e| format!("Failed to get default output config: {}", e))?;

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => Self::build_stream::<f32>(&device, &config.into(), audio_buffer),
            cpal::SampleFormat::I16 => Self::build_stream::<i16>(&device, &config.into(), audio_buffer),
            cpal::SampleFormat::U16 => Self::build_stream::<u16>(&device, &config.into(), audio_buffer),
            _ => Err("Unsupported sample format".to_string()),
        }?;

        stream
            .play()
            .map_err(|e| format!("Failed to play stream: {}", e))?;

        Ok(Self { _stream: stream })
    }

    fn build_stream<T>(
        device: &Device,
        config: &StreamConfig,
        audio_buffer: Arc<Mutex<Vec<f32>>>,
    ) -> Result<Stream, String>
    where
        T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
    {
        let channels = config.channels as usize;

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    Self::write_data(data, channels, &audio_buffer);
                },
                |err| eprintln!("Audio stream error: {}", err),
                None,
            )
            .map_err(|e| format!("Failed to build output stream: {}", e))?;

        Ok(stream)
    }

    fn write_data<T>(output: &mut [T], channels: usize, audio_buffer: &Arc<Mutex<Vec<f32>>>)
    where
        T: cpal::Sample + cpal::FromSample<f32>,
    {
        if let Ok(mut buffer) = audio_buffer.lock() {
            for frame in output.chunks_mut(channels) {
                let sample = if !buffer.is_empty() {
                    buffer.remove(0)
                } else {
                    0.0 // Silence if buffer is empty
                };

                // Write the same sample to all channels (mono to stereo/multi-channel)
                for channel_sample in frame.iter_mut() {
                    *channel_sample = T::from_sample(sample);
                }
            }
        } else {
            // If we can't lock the buffer, output silence
            for sample in output.iter_mut() {
                *sample = T::from_sample(0.0);
            }
        }
    }
}
