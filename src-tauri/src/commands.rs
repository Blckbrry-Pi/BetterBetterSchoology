use std::{collections::HashMap, sync::Arc};

use bbs_shared::{ data::ClassEntry, ClassID };
use keyring::Entry;
use reqwest_cookie_store::CookieStoreMutex;
use tauri::State;
use reqwest::{Client, Method};

use crate::{requests::{get_login_page, Selectors, login, make_api_request, get_single_class}, Credentials, structs::ActiveClasses};

#[tauri::command]
pub async fn set_credentials(creds: State<'_, Credentials>, username: String, password: String) -> Result<(), String> {
    match (creds.username.lock(), creds.password.lock()) {
        (Ok(mut username_lock), Ok(mut password_lock)) => {
            username_lock.clone_from(&Arc::new(username));
            password_lock.clone_from(&Arc::new(password));
            Ok(())
        },

        (Err(username_error), Ok(_)) => {
            eprintln!("Failed to get lock on username: {:#?}", username_error);
            Err("CredSetErr".to_string())
        },

        (Ok(_), Err(password_error)) => {
            eprintln!("Failed to get lock on password: {:#?}", password_error);
            Err("CredSetErr".to_string())
        },

        (Err(username_error), Err(password_error)) => {
            eprintln!("Failed to get lock on username and password: {:#?} {:#?}", username_error, password_error);
            Err("CredSetErr".to_string())
        },
    }
}


#[tauri::command]
pub async fn get_class_listing(
    client: State<'_, Client>,
    selectors: State<'_, Selectors>,
    creds: State<'_, Credentials>,
    cookie_jar: State<'_, Arc<CookieStoreMutex>>,
    keyring_entry: State<'_, Entry>,
) -> Result<String, String> {
    match get_login_page(&client, &selectors).await
        .map_err(|err| err.to_string()) {
        Ok(login_form_details) => {
            login(
                &client,
                creds,
                cookie_jar,
                keyring_entry,
                login_form_details,
            ).await?;
        },
        Err(_) => (),
    };

    let active_courses_text = make_api_request(
        &client,
        Method::GET,
        "/iapi/course/active",
        &HashMap::<(), ()>::new(),
    )
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let active: ActiveClasses = serde_json::from_str(&active_courses_text).map_err(|e| e.to_string())?;

    let course_listing = active.body.courses.to_by_id();

    let mut courses: Vec<_> = course_listing
        .data
        .iter()
        .map(|(nid, course)| ClassEntry {
            name: course.0.course_title.clone(),
            section: course.1.section_title.as_str().into(),
            id: ClassID(*nid),
            picture: Vec::new(),
        })
        .collect();
    
    courses.sort_unstable();

    Ok(
        base64::encode(
            bincode::serialize(&courses).map_err(|e| format!("%{}", e))?
        )
    )
}

#[tauri::command]
pub async fn parse_single_class_info(client: State<'_, Client>, classid: String) -> Result<String, String> {
    match get_single_class(&*client, classid).await {
        Ok(res) => {
            let single_class_complete = res.text().await.unwrap();
            
            println!(single_class_complete)
            Ok(body) {
                // todo make this take listing of course assignments and return it
                base64::encode(
                    bincode::serialize(&courses).map_err(|e| format!("%{}", e))?
                )

            }

            
        },
        Err(e) => Err(e.to_string()),
    }
}