use std::io::Cursor;

use anyhow::bail;
use rodio::{
    Decoder, DeviceSinkBuilder, DeviceSinkError, MixerDeviceSink, Source, decoder::DecoderError,
};

pub(crate) struct RodioAudioClient {
    output: MixerDeviceSink,
    pub volume: Volume,
}

impl RodioAudioClient {
    pub(crate) fn new() -> Result<Self, DeviceSinkError> {
        let mut output = DeviceSinkBuilder::open_default_sink()?;

        output.log_on_drop(false);

        Ok(Self {
            output,
            volume: Volume::HALF,
        })
    }

    pub(crate) fn play_sound(&self, sound: Sound) -> Result<(), DecoderError> {
        let source = Decoder::try_from(Cursor::new(sound.into_asset()))?;
        self.output
            .mixer()
            .add(source.amplify(self.volume.into_inner()));

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

pub enum Sound {
    Start,
    Coin,
    // TODO: implement sfx
    Pause,
    Unpause,
    Impact,
}
impl Sound {
    pub(crate) fn into_asset(self) -> &'static [u8] {
        match self {
            Self::Start => START,
            Self::Coin => SOUND,
            Self::Pause => PAUSE_SOUND,
            Self::Unpause => UNPAUSE_SOUND,
            Self::Impact => IMPACT_SOUND,
        }
    }
}

static START: &[u8] = include_bytes!("../../../../assets/start.wav");
static SOUND: &[u8] = include_bytes!("../../../../assets/coin.wav");
static PAUSE_SOUND: &[u8] = include_bytes!("../../../../assets/pause.wav");
static UNPAUSE_SOUND: &[u8] = include_bytes!("../../../../assets/unpause.wav");
static IMPACT_SOUND: &[u8] = include_bytes!("../../../../assets/impact.wav");
