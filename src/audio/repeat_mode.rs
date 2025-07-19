// src/audio/repeat_mode.rs
/// Define os modos de repetição possíveis
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum RepeatMode {
    None, // Não repete
    One,  // Repete a música atual
    All,  // Repete toda a playlist
}

impl RepeatMode {
    /// Alterna para o próximo modo de repetição na sequência
    pub fn next(&self) -> Self {
        match self {
            RepeatMode::None => RepeatMode::One,
            RepeatMode::One => RepeatMode::All,
            RepeatMode::All => RepeatMode::None,
        }
    }
}
