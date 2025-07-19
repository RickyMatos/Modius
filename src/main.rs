// src/main.rs
mod audio;
mod ui;
slint::include_modules!(); // Inclui os módulos do Slint

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    ui::rust::interface::gui()?; // Chama a função 'gui' que configura a interface
    Ok(())
}
