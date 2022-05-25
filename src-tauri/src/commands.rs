use std::{collections::HashMap, sync::Arc};

use bbs_shared::{ data::ClassEntry, ClassID };
use keyring::Entry;
use reqwest_cookie_store::CookieStoreMutex;
use tauri::State;
use reqwest::{Client, Method};

use crate::{requests::{get_login_page, Selectors, login, make_api_request}, Credentials, structs::ActiveClasses};



#[tauri::command]
pub async fn set_credentials(creds: State<'_, Credentials>, username: String, password: String) -> Result<(), ()> {
    creds.username.lock().unwrap().clone_from(&Arc::new(username));
    creds.password.lock().unwrap().clone_from(&Arc::new(password));
    Ok(())
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
