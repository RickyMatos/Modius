// src/audio/volume_control.rs
use crate::audio::player::AudioPlayer;
use std::sync::atomic::Ordering;

impl AudioPlayer {
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
