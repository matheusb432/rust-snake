use std::io::Cursor;

use anyhow::bail;
use rodio::{
    Decoder, DeviceSinkBuilder, DeviceSinkError, MixerDeviceSink, Source, decoder::DecoderError,
};

// TODO map enum to play sounds, use a nicer path definition
static SOUND: &[u8] = include_bytes!("../../../../assets/coin.wav");

pub(crate) struct RodioAudioClient {
    output: MixerDeviceSink,
}

impl RodioAudioClient {
    pub(crate) fn new() -> Result<Self, DeviceSinkError> {
        let output = DeviceSinkBuilder::open_default_sink()?;

        Ok(Self { output })
    }

    pub(crate) fn play_sound(&self, volume: Volume) -> Result<(), DecoderError> {
        let source = Decoder::try_from(Cursor::new(SOUND))?;
        self.output.mixer().add(source.amplify(volume.into_inner()));

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Volume(f32);
impl Volume {
    pub const FULL: Self = Self(1.0);
    pub const HALF: Self = Self(0.5);
    pub const SILENT: Self = Self(0.0);
    pub const MIN: f32 = 0_f32;
    pub const MAX: f32 = 1_f32;

    pub fn new(value: f32) -> anyhow::Result<Self> {
        if value < Self::MIN {
            bail!("volume too low, min is {}", Self::MIN)
        } else if value > Self::MAX {
            bail!("volume too high, max is {}", Self::MAX)
        }
        Ok(Self(value))
    }

    pub fn into_inner(self) -> f32 {
        self.0
    }
}
