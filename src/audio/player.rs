//src/audio/player.rs
use crate::audio::repeat_mode::RepeatMode;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::{
    error::Error,
    fs::File,
    io::BufReader,
    path::PathBuf,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering as AtomicOrdering},
    },
    time::{Duration, Instant},
};

// Wrapper thread-safe para OutputStream
pub struct OutputStreamHolder {
    pub _stream: OutputStream,
    pub stream_handle: rodio::OutputStreamHandle,
}

// Estrutura principal do player
pub struct AudioPlayer {
    pub sink: Mutex<Sink>,
    pub stream_holder: Box<OutputStreamHolder>,
    pub playlist: Mutex<Vec<PathBuf>>,
    pub current_index: Mutex<usize>,
    pub is_playing: AtomicBool,
    pub repeat_mode: Mutex<RepeatMode>,
    pub is_muted: AtomicBool,
    pub volume: Mutex<f32>,

    started_at: Mutex<Option<Instant>>,
    paused_at: Mutex<Option<Instant>>,
    paused_duration: Mutex<Duration>,
}

impl AudioPlayer {
    pub fn setup_audio() -> Result<Self, Box<dyn Error>> {
        // Obtemos tanto o stream quanto seu handle
        let (_stream, stream_handle) = OutputStream::try_default()?;

        let sink = Sink::try_new(&stream_handle)?;
        sink.set_volume(1.0);

        let stream_holder = Box::new(OutputStreamHolder {
            _stream,
            stream_handle,
        });

        Ok(Self {
            sink: Mutex::new(sink),
            stream_holder,
            playlist: Mutex::new(Vec::new()),
            current_index: Mutex::new(0),
            is_playing: AtomicBool::new(false),
            repeat_mode: Mutex::new(RepeatMode::None),
            is_muted: AtomicBool::new(false),
            volume: Mutex::new(1.0),

            // — campos de tempo —
            started_at: Mutex::new(None),
            paused_at: Mutex::new(None),
            paused_duration: Mutex::new(Duration::ZERO),
        })
    }

    // Método para acessar is_playing
    #[allow(dead_code)]
    pub fn is_playing(&self) -> bool {
        self.is_playing.load(AtomicOrdering::Relaxed)
    }

    pub fn get_total_duration(&self) -> u64 {
        let playlist = self.playlist.lock().unwrap();
        let idx = *self.current_index.lock().unwrap();
        if let Some(pathbuf) = playlist.get(idx) {
            if let Some(dur) = crate::audio::time_control::get_audio_duration_symphonia(
                pathbuf.to_str().unwrap_or_default(),
            ) {
                return dur.as_secs();
            }
        }
        0
    }

    pub fn seek_to(&self, position: u64) -> Result<(), Box<dyn Error>> {
        let playlist = self.playlist.lock().unwrap();
        let current_index = self.current_index.lock().unwrap();

        if let Some(path) = playlist.get(*current_index).cloned() {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            let source = Decoder::new(reader)?;

            let mut sink = self.sink.lock().unwrap();
            *sink = Sink::try_new(&self.stream_holder.stream_handle)?;
            //*sink = self.create_new_sink()?;

            let skipped_source = source.skip_duration(Duration::from_secs(position));
            sink.append(skipped_source);

            if self.is_playing.load(AtomicOrdering::Relaxed) {
                sink.play();
            }
        }
        Ok(())
    }

    pub fn get_current_time(&self) -> u64 {
        let started_at = self.started_at.lock().unwrap();
        let paused_at = self.paused_at.lock().unwrap();
        let paused_duration = self.paused_duration.lock().unwrap();

        if let Some(start) = *started_at {
            let now = Instant::now();
            let played = if let Some(paused) = *paused_at {
                paused.duration_since(start)
            } else {
                now.duration_since(start)
            };

            return played
                .checked_sub(*paused_duration)
                .unwrap_or_default()
                .as_secs();
        }
        0
    }

    // Método para criar uma nova Sink
    // pub fn create_new_sink(&self) -> Result<Sink, Box<dyn Error>> {
    //     Ok(Sink::try_new(&self.stream_holder.stream_handle)?)
    // }
}
