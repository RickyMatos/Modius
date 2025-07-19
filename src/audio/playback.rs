// src/audio/playback.rs
use crate::audio::player::AudioPlayer;
use crate::audio::repeat_mode::RepeatMode;
use rodio::{Decoder, Sink};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::sync::atomic::Ordering;

impl AudioPlayer {
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

            //*sink = self.create_new_sink()?;

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
            RepeatMode::One => {
                // No modo RepeatOne, apenas reinicia a música atual
                drop(repeat_mode);
                drop(playlist);
                drop(current_index);
                self.play_current()?;
            }
            RepeatMode::All => {
                // Avança para a próxima, voltando ao início se necessário
                *current_index = (*current_index + 1) % playlist.len();
                drop(repeat_mode);
                drop(playlist);
                drop(current_index);
                self.play_current()?;
            }
            RepeatMode::None => {
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
        } else if *repeat_mode == RepeatMode::All {
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
        *repeat_mode = repeat_mode.next();

        println!("Modo de repetição atualizado para: {:?}", *repeat_mode);
    }
}
