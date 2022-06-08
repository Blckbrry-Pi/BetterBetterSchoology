use std::{collections::HashMap, sync::Arc, time::SystemTime};

use bbs_shared::{ data::ClassEntry, ClassID, cache::{BackendCache, CacheDataState} };
use keyring::Entry;
use tauri::State;
use reqwest::Method;

use crate::{requests::{get_login_page, login, make_api_request}, Credentials, structs::{ActiveClasses, AugClient}};



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
        client,
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
        .chain([ClassEntry { name: "Bleep".into(), section: "P(A-D,E)".into(), id: ClassID(123456), picture: Vec::new() }].into_iter())
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

// #[tauri::command]
// pub async fn parse_single_class_info(client: State<'_, Client>, class_id: String) -> Result<ClassEntry, String> {
//     match get_single_class(&client, class_id).await {
//         Ok(res) => {
//             let body = res.text().await.unwrap();
//             println!("{}", body);

//             // let selectors = Selectors::default();
//             // let class_name = body.split(selectors.class_name).nth(1).unwrap().split(selectors.class_name_end).next().unwrap();
//             // let class_id = body.split(selectors.class_id).nth(1).unwrap().split(selectors.class_id_end).next().unwrap();
//             // Ok(ClassEntry {
//             //     class_id: ClassID::from_str(class_id).unwrap(),
//             //     class_name: class_name.to_owned(),
//             // })
            
//             unimplemented!();
//         },
//         Err(e) => Err(e.to_string()),
//     }
// }
