
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Arc;

use bbs_shared::cache::BackendCache;
use keyring::Entry;
use reqwest::Client;
use cookie_store::{CookieStore, Cookie};
use reqwest_cookie_store::CookieStoreMutex;
use app::{commands::*, Credentials, structs::AugClient};

#[cfg(debug_assertions)]
const ENABLE_KEYRING: bool = false;

#[cfg(not(debug_assertions))]
const ENABLE_KEYRING: bool = true;

fn main() {
    let keyring_entry = ENABLE_KEYRING.then(|| Entry::new("dev.skyc.betterbetterschoology.cookies", "default"));
    let mut raw_cookie_store = CookieStore::default();

    if let Some(keyring_entry) = &keyring_entry {
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
    }
    let mutable_cookie_store = CookieStoreMutex::new(raw_cookie_store);
    let final_cookie_jar = Arc::new(mutable_cookie_store);

    let final_cookie_jar_state  = final_cookie_jar.clone();
    let final_cookie_jar_client = final_cookie_jar.clone();
    drop(final_cookie_jar);

    let client = Client::builder().cookie_provider(final_cookie_jar_client).build().unwrap();

    let augmented_client = AugClient {
        client,
        cookies: final_cookie_jar_state
    };

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_class_listing, set_credentials, parse_single_class_info])
        .manage(augmented_client)
        .manage(Credentials::default())
        .manage(keyring_entry)
        .manage(BackendCache::default())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");


    // parse_single_class_info(client.into(), "5271245315".to_string());
}

