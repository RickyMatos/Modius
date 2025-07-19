mod playback;
pub mod player;
mod playlist;
mod repeat_mode;
mod time_control;
mod volume_control;

// Re-export apenas o que precisa ser visível externamente
pub use player::AudioPlayer;
//pub use repeat_mode::RepeatMode;
