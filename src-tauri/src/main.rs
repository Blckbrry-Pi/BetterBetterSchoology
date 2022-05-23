
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use reqwest::Client;

use app::{commands::*, requests::Selectors, Credentials};

fn main() {
    let client = Client::builder().cookie_store(true).build().unwrap();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_class_listing, set_credentials])
        .manage(client)
        .manage(Selectors::default())
        .manage(Credentials::default())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


