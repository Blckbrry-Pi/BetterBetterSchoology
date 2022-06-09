use std::{collections::HashMap, sync::Arc, time::SystemTime};

use bbs_shared::{ data::ClassEntry, ClassID, cache::{BackendCache, CacheDataState}, SectionID };
use keyring::Entry;
use tauri::State;
use reqwest::Method;

use crate::{requests::{get_login_page, login, make_api_request, get_single_class}, Credentials, structs::{ActiveClasses, AugClient}};

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
    aug_client: State<'_, AugClient>,
    creds: State<'_, Credentials>,
    cache: State<'_, BackendCache>,
    keyring_entry: State<'_, Option<Entry>>,
) -> Result<String, String> {
    let client = &aug_client.client;
    let cookie_jar = &aug_client.cookies;

    if cache.get_class_listing_state() == CacheDataState::Ok {
        if let Some(guard) = cache.class_listing.data.try_lock().ok() {
            if let Some(courses) = guard.as_ref() {
                return Ok(
                    base64::encode(
                        bincode::serialize(courses).map_err(|e| format!("%{}", e))?
                    )
                );
            }
        }
    }
    
    
    match get_login_page(client).await
        .map_err(|err| err.to_string()) {
        Ok(login_form_details) => {
            login(
                client,
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

    
    let active: ActiveClasses = serde_json::from_str(active_courses_text.as_ref()).map_err(|e| e.to_string())?;

    let course_listing = active.body.courses.to_by_id();

    let mut courses: Vec<_> = course_listing
        .data
        .iter()
        .map(|(nid, (course, section))| ClassEntry {
            name: course.course_title.clone(),
            section: section.section_title.as_str().into(),
            id: ClassID(*nid),
            section_nid: SectionID(section.nid),
            picture: Vec::new(),
            instructors: None,
        })
        .chain([ClassEntry {
            name: "Test Class with Bad Section".into(),
            section: "P(A-D,E)".into(),
            id: ClassID(123456),
            section_nid: SectionID(654321),
            picture: Vec::new(),
            instructors: None
        }].into_iter())
        .collect();
    
    courses.sort_unstable();

    println!("Testpoint 1");
    
    let encoded_output = base64::encode(
        bincode::serialize(&courses).map_err(|e| format!("%{}", e))?
    );

    println!("Testpoint 2");

    match (cache.class_listing.prev_update.lock(), cache.class_listing.data.lock()) {
        (Ok(mut prev_update), Ok(mut class_listing)) => {
            *prev_update = SystemTime::now();
            *class_listing = Some(courses);
        },
        (
            update_res,
            data_res,
        ) => eprintln!("Cache lock poisoned: {:#?}\n{:#?}", update_res, data_res),
    }

    println!("Testpoint 3");

    Ok(encoded_output)
}

#[tauri::command]
pub async fn parse_single_class_info(client: State<'_, AugClient>, classid: String) -> Result<String, String> {
    match get_single_class(&client.client, classid).await {
        Ok(res) => {
            let single_class_complete = res.text().await.unwrap();
            Ok({
                // todo make this take listing of course assignments and return it
                base64::encode(
                    bincode::serialize(&single_class_complete).map_err(|e| format!("%{}", e))?
                )

            })

            
        },
        Err(e) => Err(e.to_string()),
    }
}
