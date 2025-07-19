// src/audio/playlist.rs
use crate::audio::player::AudioPlayer;
use std::error::Error;
use std::path::PathBuf;
use std::sync::atomic::Ordering;

impl AudioPlayer {
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
        if is_first_song && !self.is_playing.load(Ordering::SeqCst) {
            self.play_current()?;
        }

        Ok(())
    }
}
