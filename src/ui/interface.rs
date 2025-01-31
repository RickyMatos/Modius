// src/ui/interface.rs
use crate::audio::player::AudioPlayer;
use slint::{ComponentHandle, TimerMode, Weak};
use std::error::Error;
use std::sync::{Arc, Mutex};
//use std::thread;

slint::include_modules!();

/*------THREAD DE REPRODUÇÃO------*/
// fn playing_thread(player: Arc<Mutex<AudioPlayer>>, main_window: Weak<MainWindow>) {
//     thread::spawn(move || {
//         while let Some(window) = main_window.upgrade() {
//             if let Ok(player) = player.lock() {
//                 let current_time = player.get_current_time();
//                 let total_duration = player.get_total_duration();

//                 let current_formatted = format_time(current_time);
//                 let total_formatted = format_time(total_duration);

//                 window.set_current_time(current_formatted.into());
//                 window.set_total_time(total_formatted.into());

//                 if total_duration > 0 {
//                     let progress = current_time as f32 / total_duration as f32;
//                     window.set_progress(progress);
//                 }
//             }
//             thread::sleep(std::time::Duration::from_millis(1000));
//         }
//         println!("Thread de reprodução encerrada.");
//     });
// }

/*------FIM DA THREAD DE REPRODUÇÃO------*/

/*-------Função para a contagem de tempo------*/
fn format_time(seconds: u64) -> String {
    let minutes = seconds / 60;
    let seconds = seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}
/*-------FIM PARA A CONTAGEM DE TEMPO------*/

/*------THREAD DOS CONTROLES------*/
// pub fn buttons_thread(player: Arc<Mutex<AudioPlayer>>, main_window: Weak<MainWindow>) {
//     thread::spawn(move || {
//         while let Some(window) = main_window.upgrade() {
//             if let Ok(player) = player.lock() {
//                 window.set_is_playing(player.is_playing());
//                 window.set_volume(player.get_volume() as f32);
//                 window.set_is_muted(player.is_muted());
//                 // window.set_playlist(player.playlist());
//             }
//             thread::sleep(std::time::Duration::from_millis(100));
//         }
//         println!("Thread dos botões encerrada.");
//     });
// }

/*------FIM DA THREAD DOS CONTROLES------*/

// Função para callbacks do Slint
pub fn setup_callbacks(player: Arc<Mutex<AudioPlayer>>, main_window: &MainWindow) {
    // Função para o volume
    pub fn volume(player: Arc<Mutex<AudioPlayer>>, main_window: &MainWindow) {
        let button_volume = player.clone();
        main_window.on_volume_changed(move |value| {
            if let Ok(mut player) = button_volume.lock() {
                player.set_volume(value as f32);
                println!("Volume changed to: {}", value); // Para debug
            }
        });

        // Timer para atualizar o estado do volume na interface
        let button_volume_update = player.clone();
        let window_handle = main_window.as_weak();
        slint::Timer::default().start(
            TimerMode::Repeated,
            std::time::Duration::from_millis(100),
            move || {
                if let Some(window) = window_handle.upgrade() {
                    if let Ok(player) = button_volume_update.lock() {
                        window.set_volume(player.get_volume() as f32);
                        window.set_is_muted(player.is_muted());
                    }
                }
            },
        );
    }

    // Função dos botões
    pub fn buttons(player: Arc<Mutex<AudioPlayer>>, main_window: &MainWindow) {
        // Botão para adicionar musica a playlist
        let button_add_playlist = player.clone();
        main_window.on_open_file_manager(move || {
            println!("Abrindo gerenciador de arquivos...");
            let file = rfd::FileDialog::new()
                .add_filter("MP3 Files", &["mp3"])
                .pick_file();

            if let Some(selected_file) = file {
                println!("Arquivo selecionado: {:?}", selected_file);
                if let Ok(mut player) = button_add_playlist.lock() {
                    match player.playlist(selected_file) {
                        Ok(_) => println!("Arquivo adicionado com sucesso à playlist"),
                        Err(e) => eprintln!("Erro ao adicionar arquivo: {}", e),
                    }
                }
            }
        });

        // Botão do play
        let button_play = player.clone();
        main_window.on_play_clicked(move || {
            println!("Botão de play/pause clicado");
            if let Ok(mut player) = button_play.lock() {
                player.toggle_playback();
            }
        });

        // Botão do proximo
        let button_next = player.clone();
        main_window.on_next_clicked(move || {
            println!("Botão next clicado.");
            if let Ok(mut player) = button_next.lock() {
                if let Err(e) = player.next() {
                    eprintln!("Erro ao avançar música: {}", e);
                }
            }
        });

        // Botão do anterior
        let button_previous = player.clone();
        main_window.on_previous_clicked(move || {
            println!("Botão previous clicado.");
            if let Ok(mut player) = button_previous.lock() {
                if let Err(e) = player.previous() {
                    eprintln!("Erro ao voltar música: {}", e);
                }
            }
        });

        // Botão da repetição
        let button_repeat = player.clone();
        main_window.on_repeat_clicked(move || {
            println!("Botão repeat clicado.");
            if let Ok(mut player) = button_repeat.lock() {
                player.repeats();
            }
        });

        // Butão para mutar
        let button_mute = player.clone();
        main_window.on_mute_toggled(move || {
            if let Ok(player) = button_mute.lock() {
                player.toggle_mute();
            }
        });
    }

    // Função da barra de progesso
    pub fn progress_bar(player: Arc<Mutex<AudioPlayer>>, main_window: &MainWindow) {
        // Barra de Progresso
        let progress_bar = player.clone();
        main_window.on_time_selected(move |value| {
            if let Ok(player) = progress_bar.lock() {
                let total_duration = player.get_total_duration();
                let new_time = (value * total_duration as f32) as u64;
                if let Err(e) = player.seek_to(new_time) {
                    println!("Erro ao buscar posição: {}", e);
                }
            }
        });

        let player_time = player.clone();
        let window_handle = main_window.as_weak();
        slint::Timer::default().start(
            TimerMode::Repeated,
            std::time::Duration::from_millis(1000),
            move || {
                let window = window_handle.unwrap();
                if let Ok(player) = player_time.lock() {
                    let current_time = player.get_current_time();
                    let total_duration = player.get_total_duration();

                    let current_formatted = format_time(current_time);
                    let total_formatted = format_time(total_duration);

                    window.set_current_time(current_formatted.into());
                    window.set_total_time(total_formatted.into());

                    if total_duration > 0 {
                        let progress = current_time as f32 / total_duration as f32;
                        window.set_progress(progress);
                    }
                }
            },
        );
    }

    buttons(player.clone(), main_window);
    volume(player.clone(), main_window);
    progress_bar(player.clone(), main_window);
}

// Função da interface
pub fn gui() -> Result<(), Box<dyn Error>> {
    println!("Inicializando a interface gráfica...");

    // Iniciar os recursos de audio
    let player = Arc::new(Mutex::new(AudioPlayer::setup_audio()?));
    // Inicia a janela
    let main_window = MainWindow::new()?;

    // Configura os callbacks
    setup_callbacks(player.clone(), &main_window);

    // Inicia as threads
    //let window_handle = main_window.as_weak();
    //buttons_thread(player.clone(), window_handle.clone());
    //playing_thread(player.clone(), window_handle);

    main_window.run()?;
    Ok(())
}
