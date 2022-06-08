
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Arc;

use keyring::Entry;
use reqwest::Client;
use cookie_store::{CookieStore, Cookie};
use reqwest_cookie_store::CookieStoreMutex;
use app::{commands::*, requests::Selectors, Credentials};

fn main() {
    let keyring_entry = Entry::new("dev.skyc.betterbetterschoology.cookies", "default");
    let mut raw_cookie_store = CookieStore::default();

    if let Ok(json) = keyring_entry.get_password() {
        if let Ok(store) = bincode::deserialize::<Vec<Cookie>>(&base64::decode(json).unwrap()) {
            store
                .iter()
                .filter_map(|cookie| raw_cookie_store
                    .insert(cookie.clone(), &url::Url::parse(&format!("https://{}", cookie.domain.as_cow().unwrap())).unwrap())
                    .err()
                    .map(|error| (cookie, error))
                )
                .for_each(|(cookie, error)| println!("Failed to insert cookie {:?}! Failed with: {}", cookie, error));
        }
    }
    let mutable_cookie_store = CookieStoreMutex::new(raw_cookie_store);
    let final_cookie_jar = Arc::new(mutable_cookie_store);

    let final_cookie_jar_state  = final_cookie_jar.clone();
    let final_cookie_jar_client = final_cookie_jar.clone();
    drop(final_cookie_jar);


    
    
    let client = Client::builder().cookie_provider(final_cookie_jar_client).build().unwrap();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_class_listing, set_credentials, parse_single_class_info])
        .manage(client)
        .manage(Selectors::default())
        .manage(Credentials::default())
        .manage(final_cookie_jar_state)
        .manage(keyring_entry)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


