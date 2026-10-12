// * * * PUNTO DE ENTRADA PRINCIPAL EN RUST WEBASSEMBLY (WASM) * * *

mod app;
mod components;
mod models;

use app::App;
use leptos::*;

fn main() {
    // Configura captura de pánicos para volcado legible en la consola del navegador
    console_error_panic_hook::set_once();
    
    // Inicializa el logger WASM
    let _ = console_log::init_with_level(log::Level::Debug);

    log::info!("Iniciando Tigo SOC Frontend SPA (Leptos WASM)...");

    // Monta la aplicación Leptos directamente en el elemento body del DOM
    mount_to_body(App);
}
