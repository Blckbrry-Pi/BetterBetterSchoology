use std::{error::Error, collections::HashMap, borrow::Cow, fmt::Display, ops::Deref, sync::Arc};

use bbs_shared::errors::LoginError;
use keyring::Entry;
use reqwest::{Client, Response, Method};
use reqwest_cookie_store::CookieStoreMutex;
use scraper::{Html, Selector};
use serde::Serialize;
use tauri::State;
use derive_getters::Getters;

use crate::Credentials;

lazy_static::lazy_static! {
    static ref LOGIN_FORM: Selector = Selector::parse("form#s-user-login-form").unwrap();
    static ref LOGIN_INPUT: Selector = Selector::parse("input").unwrap();
}



#[derive(Getters, Debug, Clone)]
pub struct LoginFormDetails {
    method: String,
    action: String,
    inputs: Vec<HashMap<String, String>>,
}

#[derive(Debug)]
pub struct NotFoundError {
    value: Cow<'static, str>,
}
impl Display for NotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(&self.value)
    }
}

impl Error for NotFoundError {}

pub async fn get_login_page(client: &Client) -> Result<LoginFormDetails, LoginError> {
    use bbs_shared::errors::LoginError::*;

    let output = client
        .get("https://bca.schoology.com")
        .send()
        .await;
    let res = output.or(Err(RequestError))?;
    let text = res.text().await.or(Err(DecodeError))?;
    let document = Html::parse_document(&text);
    
    let forms = document.select(&LOGIN_FORM);

    let form_node = match forms.last() {
        Some(node) => node,
        None => return Err(FindFormError),
    };

    let method = form_node.value().attr("method").unwrap().to_owned();
    let action = form_node.value().attr("action").unwrap().to_owned();

    let inputs = form_node
        .select(&LOGIN_INPUT)
        .map(|element_ref| element_ref.value())
        .map(|element| element
            .attrs()
            .map(|(key, value)| (key.to_owned(), value.to_owned()))
            .collect()
        )
        .collect();
        
    Ok(LoginFormDetails {
        inputs,
        action,
        method,
    })
}

pub async fn login(
    client: &Client,
    creds: State<'_, Credentials>,
    cookie_jar: &Arc<CookieStoreMutex>,
    keyring_entry: State<'_, Option<Entry>>,
    login_form_details: LoginFormDetails,
) -> Result<(), LoginError> {
    use LoginError::*;

    let form: Vec<(String, String)> = login_form_details
        .inputs()
        .iter()
        .filter_map(|element| Some((element.get("name")?, element)))
        .filter(|(name, _)| name.as_str() != "op")
        .filter_map(
            |(name, input)| match input.get("value") {
                Some(value) if value != "" => Some((name.to_owned(), value.to_owned())),
                _ => if name.contains("pass") {
                    Some((
                        name.to_owned(),
                        creds.password
                            .lock()
                            .ok()
                            .map(|mutex_guard| (*mutex_guard).deref().clone())
                            .unwrap_or_default()
                    ))
                } else if name.contains("mail") {
                    Some((
                        name.to_owned(),
                        creds.username
                            .lock()
                            .ok()
                            .map(|mutex_guard| (*mutex_guard).deref().clone())
                            .unwrap_or_default()
                    ))
                } else {
                    None
                }
            }
        )
        .collect();
        
    let response = match login_form_details.method().as_str() {
        "post" => client.post(format!("https://bca.schoology.com{}", login_form_details.action())).form(&form),
        "get"  => client.get (format!("https://bca.schoology.com{}", login_form_details.action())),
        _ => unimplemented!("Invalid method form method: `{}`!", login_form_details.method()),
    }
        .send()
        .await;
    
    

    match response {
        Ok(res) => {
            let status = res.status();


            if status.is_success() {
                let text = res.text().await.or(Err(DecodeError))?;
                let index = text.find("aria-invalid");
                if index.is_some() && text[index.unwrap()..index.unwrap() + 100].contains("unrecognized") {
                    Err(InvalidCredsError)
                } else {
                    match cookie_jar.lock() {
                        Ok(inner_jar) => {
                            match bincode::serialize(&inner_jar.iter_unexpired().collect::<Vec<_>>()) {
                                Ok(serialized_value)  => {
                                    let base_64_value = base64::encode(serialized_value);
                                    if let Some(keyring_entry) = &*keyring_entry {
                                        if let Err(e) = keyring_entry.set_password(&base_64_value) {
                                            eprintln!("Keyring failed to save cookies: {}", e);
                                        }
                                    }
                                }
                                Err(e) => eprintln!("Failed to serialize cookies into binary: {}", e),
                            }
                        },
                        Err(e) => eprintln!("Failed to get lock on cookie jar: {}", e),
                    }
                    Ok(())
                }
            } else {
                Err(LaterRequestError)
            }
        },
        Err(_) => Err(LaterRequestError),
    }
}

pub async fn make_api_request<T>(
    client: &Client,
    method: Method,
    route: &str,
    body: &T,
) -> Result<Response, reqwest::Error>
where 
    T: Serialize + ?Sized
{
    client
        .request(method, format!("https://bca.schoology.com{}", route))
        .form(body)
        .send()
        .await
}

pub async fn get_single_class(client: &Client, classid: String) -> Result<Response, reqwest::Error> {
    client.get(format!("https://bca.schoology.com/course/{}/materials", classid)).send().await
}

pub async fn get_assignment_page(client: &Client, assignmentid: String) -> Result<Response, reqwest::Error> {
    client.get(format!("https://bca.schoology.com/assignment/{}/info", assignmentid)).send().await
}