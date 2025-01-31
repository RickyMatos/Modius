// src/audio/player.rs
use rodio::{Decoder, OutputStream, Sink, Source};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

// Wrapper thread-safe para OutputStream
struct OutputStreamHolder {
    _stream: OutputStream,
    stream_handle: rodio::OutputStreamHandle,
}

// Implementação de Send e Sync manualmente para o holder
unsafe impl Send for OutputStreamHolder {}
unsafe impl Sync for OutputStreamHolder {}

// atributos para os metodos do reprodutor
pub struct AudioPlayer {
    sink: Mutex<Sink>,
    stream_holder: Box<OutputStreamHolder>,
    playlist: Mutex<Vec<PathBuf>>,
    current_index: Mutex<usize>,
    is_playing: AtomicBool,
    repeat_mode: Mutex<RepeatMode>,
    pub is_muted: AtomicBool,
    pub volume: Mutex<f32>,
}

// Define os modos de repetição possíveis
#[derive(PartialEq, Debug)]
pub enum RepeatMode {
    NoRepeat,  // Não repete
    RepeatOne, // Repete a música atual
    RepeatAll, // Repete toda a playlist
}

//
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
            repeat_mode: Mutex::new(RepeatMode::NoRepeat),
            is_muted: AtomicBool::new(false),
            volume: Mutex::new(1.0),
        })
    }

    //  método para acessar is_playing
    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::Relaxed)
    }

    // Função para adicionar música à playlist
    pub fn playlist(&mut self, path: PathBuf) -> Result<(), Box<dyn Error>> {
        println!("Adicionando música à playlist: {:?}", path);

        let mut playlist = self.playlist.lock().unwrap();
        let is_first_song = playlist.is_empty();
        playlist.push(path);
        let playlist_len = playlist.len();
        drop(playlist); // Liberamos o lock antes de chamar play_current

        println!("Tamanho atual da playlist: {}", playlist_len);

        // Só iniciamos a reprodução se for a primeira música e não estiver tocando
        if is_first_song && !self.is_playing() {
            self.play_current()?;
        }

        Ok(())
    }

    // Função para reproduzir arquivo atual
    pub fn play_current(&mut self) -> Result<(), Box<dyn Error>> {
        let playlist = self.playlist.lock().unwrap();
        let current_index = self.current_index.lock().unwrap();

        if playlist.is_empty() {
            println!("Playlist vazia");
            return Ok(());
        }

        if *current_index >= playlist.len() {
            println!("Índice atual inválido");
            return Ok(());
        }

        if let Some(path) = playlist.get(*current_index) {
            println!("Reproduzindo música: {:?}", path);

            // Criamos uma nova sink para garantir um estado limpo
            let mut sink = self.sink.lock().unwrap();
            *sink = Sink::try_new(&self.stream_holder.stream_handle)?;

            let file = File::open(path)?;
            let reader = BufReader::new(file);
            let source = Decoder::new(reader)?;

            sink.append(source);
            sink.play();
            self.is_playing.store(true, Ordering::Relaxed);

            println!("Estado de reprodução: playing");
        } else {
            println!("Nenhuma música disponível para reprodução");
        }
        Ok(())
    }

    // Método play/pause
    pub fn toggle_playback(&mut self) {
        println!(
            "Toggle playback chamado. Estado atual: {}",
            if self.is_playing.load(Ordering::Relaxed) {
                "tocando"
            } else {
                "pausado"
            }
        );

        if self.playlist.lock().unwrap().is_empty() {
            println!("Playlist vazia, nada para reproduzir");
            return;
        }

        let sink = self.sink.lock().unwrap();
        if self.is_playing.load(Ordering::Relaxed) {
            sink.pause();
            self.is_playing.store(false, Ordering::Relaxed);
            println!("Música pausada");
        } else {
            sink.play();
            self.is_playing.store(true, Ordering::Relaxed);
            println!("Música resumida");
        }
    }
    // Função para avançar musica pra proxima
    pub fn next(&mut self) -> Result<(), Box<dyn Error>> {
        let playlist = self.playlist.lock().unwrap();
        if playlist.is_empty() {
            return Ok(());
        }

        let repeat_mode = self.repeat_mode.lock().unwrap();
        let mut current_index = self.current_index.lock().unwrap();

        match *repeat_mode {
            RepeatMode::RepeatOne => {
                // No modo RepeatOne, apenas reinicia a música atual
                drop(repeat_mode);
                drop(playlist);
                drop(current_index);
                self.play_current()?;
            }
            RepeatMode::RepeatAll => {
                // Avança para a próxima, voltando ao início se necessário
                *current_index = (*current_index + 1) % playlist.len();
                drop(repeat_mode);
                drop(playlist);
                drop(current_index);
                self.play_current()?;
            }
            RepeatMode::NoRepeat => {
                // Só avança se houver próxima música
                if *current_index < playlist.len() - 1 {
                    *current_index += 1;
                    drop(repeat_mode);
                    drop(playlist);
                    drop(current_index);
                    self.play_current()?;
                }
            }
        }
        Ok(())
    }

    // Função para retorna a musica anterior
    pub fn previous(&mut self) -> Result<(), Box<dyn Error>> {
        let playlist = self.playlist.lock().unwrap();
        if playlist.is_empty() {
            return Ok(());
        }

        let mut current_index = self.current_index.lock().unwrap();
        let repeat_mode = self.repeat_mode.lock().unwrap();

        if *current_index > 0 {
            *current_index -= 1;
            drop(playlist);
            drop(current_index);
            drop(repeat_mode);
            self.play_current()?;
        } else if *repeat_mode == RepeatMode::RepeatAll {
            *current_index = playlist.len() - 1;
            drop(playlist);
            drop(current_index);
            drop(repeat_mode);
            self.play_current()?;
        }
        Ok(())
    }

    // Função para alterna entre os modos de repetição
    pub fn repeats(&mut self) {
        let mut repeat_mode = self.repeat_mode.lock().unwrap();
        *repeat_mode = match *repeat_mode {
            RepeatMode::NoRepeat => RepeatMode::RepeatOne,
            RepeatMode::RepeatOne => RepeatMode::RepeatAll,
            RepeatMode::RepeatAll => RepeatMode::NoRepeat,
        };

        println!("Modo de repetição atualizado para: {:?}", *repeat_mode);
    }

    pub fn get_current_time(&self) -> u64 {
        let playlist = self.playlist.lock().unwrap();
        let current_index = self.current_index.lock().unwrap();

        // Implementação mais precisa do tempo atual
        if let Some(path) = playlist.get(*current_index) {
            if let Ok(file) = File::open(path) {
                if let Ok(decoder) = Decoder::new(BufReader::new(file)) {
                    if let Some(duration) = decoder.total_duration() {
                        let sink = self.sink.lock().unwrap();
                        return (duration.as_secs_f64() * sink.len() as f64) as u64;
                    }
                }
            }
        }
        0
    }

    //
    pub fn get_total_duration(&self) -> u64 {
        let playlist = self.playlist.lock().unwrap();
        let current_index = self.current_index.lock().unwrap();

        if let Some(path) = playlist.get(*current_index) {
            if let Ok(file) = File::open(path) {
                if let Ok(source) = Decoder::new(BufReader::new(file)) {
                    return source.total_duration().unwrap_or_default().as_secs();
                }
            }
        }
        0 // Retorna 0 se não houver música ou ocorrer erro
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

            let skipped_source = source.skip_duration(Duration::from_secs(position));
            sink.append(skipped_source);

            if self.is_playing.load(Ordering::Relaxed) {
                sink.play();
            }
        }
        Ok(())
    }

    // Função do volume
    pub fn set_volume(&mut self, new_volume: f32) {
        let mut volume = self.volume.lock().unwrap();
        *volume = new_volume.clamp(0.0, 1.0);
        if !self.is_muted.load(Ordering::Relaxed) {
            self.sink.lock().unwrap().set_volume(*volume);
        }
    }

    pub fn get_volume(&self) -> f32 {
        *self.volume.lock().unwrap()
    }

    pub fn toggle_mute(&self) {
        let was_muted = self.is_muted.fetch_xor(true, Ordering::Relaxed);
        let volume = self.volume.lock().unwrap();
        let sink = self.sink.lock().unwrap();

        if was_muted {
            sink.set_volume(*volume);
        } else {
            sink.set_volume(0.0);
        }
    }

    pub fn is_muted(&self) -> bool {
        self.is_muted.load(Ordering::Relaxed)
    }
}
