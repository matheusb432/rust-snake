use std::io::Cursor;

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
    pub const HALF: Self = Self(0.5);

    pub fn into_inner(self) -> f32 {
        self.0
    }
}

pub enum Sound {
    Start,
    Coin,
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

static START: &[u8] = include_bytes!("../../../../assets/start.flac");
static SOUND: &[u8] = include_bytes!("../../../../assets/coin.flac");
static PAUSE_SOUND: &[u8] = include_bytes!("../../../../assets/pause.flac");
static UNPAUSE_SOUND: &[u8] = include_bytes!("../../../../assets/unpause.flac");
static IMPACT_SOUND: &[u8] = include_bytes!("../../../../assets/impact.flac");
