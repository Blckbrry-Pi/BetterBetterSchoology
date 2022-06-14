use std::{collections::HashMap, sync::Arc, time::SystemTime};

use bbs_shared::{ data::{ClassEntry, Assignment}, ClassID, cache::{BackendCache, CacheDataState}, SectionID, errors::{CredSetError, LoginError} };
use keyring::Entry;
use tauri::State;
use reqwest::Method;
use scraper::{Html, Selector};

use crate::{requests::{get_login_page, login, make_api_request, get_single_class}, Credentials, structs::{ActiveClasses, AugClient}};

#[tauri::command]
pub async fn set_credentials(creds: State<'_, Credentials>, username: String, password: String) -> Result<(), String> {
    match (creds.username.lock(), creds.password.lock()) {
        (
            Ok(mut username_lock),
            Ok(mut password_lock)
        ) => {
            username_lock.clone_from(&Arc::new(username));
            password_lock.clone_from(&Arc::new(password));
            return Ok(());
        },

        (
            Err(username_error),
            Ok(_)
        ) => eprintln!("Failed to get lock on username: {:#?}", username_error),

        (
            Ok(_),
            Err(password_error)
        ) => eprintln!("Failed to get lock on password: {:#?}", password_error),

        (
            Err(username_error),
            Err(password_error)
        ) => eprintln!("Failed to get lock on username and password: {:#?} {:#?}", username_error, password_error),
    }

    Err(CredSetError.into())
}


pub async fn is_logged_in(
    aug_client: State<'_, AugClient>,
    cache: State<'_, BackendCache>,
) -> Result<String, String> {
    use bbs_shared::errors::LoginError::SerializationError;

    let client = &aug_client.client;

    let return_bool = if cache.get_class_listing_state() == CacheDataState::Ok {
        Ok(true)
    } else {
        match get_login_page(client).await {
            Ok(_) => Ok(false),
            Err(e) => match e {
                LoginError::FindFormError => Ok(true), // TODO: Handle case where schoology is down.
                _ => Err::<_, String>(e.into()),
            },
        }
    }?;

    
    Ok(base64::encode(
        bincode
            ::serialize(&return_bool)
            .or::<String>(Err(SerializationError.into()))?,
    ))
}

#[tauri::command]
pub async fn get_class_listing(
    aug_client: State<'_, AugClient>,
    creds: State<'_, Credentials>,
    cache: State<'_, BackendCache>,
    keyring_entry: State<'_, Option<Entry>>,
) -> Result<String, String> {
    
    use bbs_shared::errors::LoginError::*;

    let client = &aug_client.client;
    let cookie_jar = &aug_client.cookies;

    if cache.get_class_listing_state() == CacheDataState::Ok {
        if let Some(guard) = cache.class_listing.data.try_lock().ok() {
            if let Some(courses) = guard.as_ref() {
                return Ok(base64::encode(
                    bincode
                        ::serialize(courses)
                        .or::<String>(Err(SerializationError.into()))?,
                ));
            }
        }
    }
    
    match get_login_page(client).await {
        Ok(login_form_details) => {
            match login(
                client,
                creds,
                cookie_jar,
                keyring_entry,
                login_form_details,
            ).await {
                Ok(_) => (),
                Err(e) => return Err(e.into())
            }
        },
        Err(FindFormError) => (),
        Err(e) => return Err(e.into()),
    };

    let active_courses_text = make_api_request(
        &client,
        Method::GET,
        "/iapi/course/active",
        &HashMap::<(), ()>::new(),
    )
        .await
        .or::<String>(Err(LaterRequestError.into()))?
        .text()
        .await
        .or::<String>(Err(DecodeError.into()))?;

    
    let active: ActiveClasses = serde_json
        ::from_str(active_courses_text.as_ref())
        .or::<String>(Err(JsonError.into()))?;

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
    
    let encoded_output = base64::encode(
        bincode
            ::serialize(&courses)
            .or::<String>(Err(SerializationError.into()))?,
    );

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

    Ok(encoded_output)
}


// TODO -- ANY ASSIGNMENTS THAT HAVE <br> </br> -- REMOVE FIRST <br> AND REPLACE END TAG WITH NEW LINE
//         can probably also figure out a way to condense the selectors --> very messy right now, but like everything else, code hard will implement later

// currently, this returns a vector of 2 vectors, with the first vector containing all assignments w/ due dates, and the second containing all files/links under a given class page
#[tauri::command]
pub async fn parse_single_class_info(
    client: State<'_, AugClient>,
    classid: String
) -> Result<String, String> {
    let tempclient = &client.client;
    match get_single_class(tempclient, classid).await {
        Ok(res) => { 
            let body = res.text().await.unwrap();
            let document = Html::parse_document(&body);

            // get all assignments for a class --> need to make document copies b/c of borrow checker
            let assign_doc = document.clone();
            let assignments = assignment_data(assign_doc);

            // get all "documents" (pdfs, word docs, etc) and links for a class
            let file_doc = document.clone();
            let files = file_data(file_doc);

            let all_materials = vec![assignments, files];

            return Ok(
                base64::encode(
                    bincode::serialize(&all_materials).map_err(|e| format!("%{}", e))?
                )
            );
        },
        Err(e) => Err(e.to_string()),
    }
}

pub fn assignment_data (document : Html) -> Vec<Assignment> {
    // selecting all assignments
    let assignment_selector = Selector::parse("tr.type-assignment").unwrap();
    let all_assignments = document.select(&assignment_selector);

    // getting specific parts of the assignment
    let title_selector = Selector::parse(".item-title>a").unwrap();
    let body_selector = Selector::parse(".item-body>p").unwrap();
    let duedate_selector = Selector::parse(".item-subtitle>span").unwrap();
    let info_selector = Selector::parse(".item-info").unwrap();

    let assignments : Vec<_> = all_assignments
        .into_iter()
        .map(|element| {
            let assignment = element.select(&info_selector).next().unwrap();
            let title = assignment.select(&title_selector).next().unwrap();
            let body = assignment.select(&body_selector).next().unwrap();
            let id = title.value().attr("href").unwrap()[12..].to_string();

            let duedate = assignment.select(&duedate_selector).next();
            let duedate = match duedate {
                Some(duedate) => duedate.inner_html(),
                None => "No Due Date Specified".to_string(),
            };

            Assignment {
                id : id,
                kind : "".to_string(),
                title: title.inner_html(),
                body: body.inner_html(),
                duedate: duedate,
            }
        })
        .collect();

        assignments
}


// may need to dbl check to see if there are no files or not -- not sure if this will cause issue
pub fn file_data (document : Html) -> Vec<Assignment> {
    let document_selector = Selector::parse("tr.type-document").unwrap();
    let all_attachments = document.select(&document_selector);
    let file_selector = Selector::parse(".attachments-file-name ").unwrap();
    let link_selector = Selector::parse(".attachments-link>a").unwrap();

    // either attachments-link or attachments-file

    let title_selector = Selector::parse("a").unwrap();
    let extra_title_selector = Selector::parse("a>span").unwrap();

    let docs : Vec<_> = all_attachments
        .into_iter()
        .map(|element| {
            let id = element.value().attr("id").unwrap()[12..].to_string();
            if element.inner_html().contains("attachments-file") {
                // file case    
                let el = element.select(&file_selector).next().unwrap();
                let mut title = el.select(&title_selector).next().unwrap();
                let actual_title : String;
                if title.inner_html().contains("<span ") {
                    title = title.select(&extra_title_selector).next().unwrap();
                    actual_title = title.inner_html().split("<span ").collect::<Vec<&str>>()[0].to_string();
                } else {
                    actual_title = title.inner_html();
                }

                Assignment {
                    id,
                    kind : "file".to_string(),
                    title: actual_title,
                    body: "".to_string(),
                    duedate : "No Due Date Specified".to_string(),
                }
            } else {
                // link case
                let el = element.select(&link_selector).next().unwrap();
                Assignment {
                    id,
                    kind : "link".to_string(),
                    title: el.inner_html(),
                    body: "".to_string(),
                    duedate : "No Due Date Specified".to_string(),
                }
            }            
        })
        .collect();
    
    docs
}
