// src/audio/time_control.rs
use std::{fs::File, time::Duration as StdDuration};
use symphonia::{
    core::{formats::FormatOptions, io::MediaSourceStream, meta::MetadataOptions, probe::Hint},
    default::get_probe,
};

/// Retorna a duração total da faixa em segundos, ou `None` em caso de falha.
pub fn get_audio_duration_symphonia(path: &str) -> Option<StdDuration> {
    // 1. Abre o arquivo
    let file = File::open(path).ok()?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    // 2. Configura opções de formato e metadados
    let hint = Hint::new();
    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();

    // 3. Prova o formato
    let probe = get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .ok()?;
    let reader = probe.format;

    // 4. Pega o primeiro track de áudio
    let track = reader.default_track()?;

    // 5. Usa n_frames e sample_rate para calcular duração
    if let (Some(frames), Some(sr)) = (track.codec_params.n_frames, track.codec_params.sample_rate)
    {
        let secs = frames as f64 / sr as f64;
        return Some(StdDuration::from_secs_f64(secs));
    }

    None
}
