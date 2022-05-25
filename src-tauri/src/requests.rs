use std::{error::Error, collections::HashMap, borrow::Cow, fmt::Display, ops::Deref, sync::Arc};

use keyring::Entry;
use reqwest::{Client, Response, Method};
use reqwest_cookie_store::CookieStoreMutex;
use scraper::{Html, Selector};
use serde::Serialize;
use tauri::State;
use derive_getters::Getters;

use crate::Credentials;

#[derive(Getters)]
pub struct Selectors {
    login_form: Selector,
    login_input: Selector,
}


impl Default for Selectors {
    fn default() -> Self {
        Self {
            login_form: Selector::parse("form#s-user-login-form").unwrap(),
            login_input: Selector::parse("input").unwrap(),
        }
    }
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

pub async fn get_login_page(client: &State<'_, Client>, selectors: &State<'_, Selectors>) -> Result<LoginFormDetails, Box<dyn Error + Send + Sync>> {
    let output = client
        .get("https://bca.schoology.com")
        .send()
        .await;
    let res = output?;
    let text = res.text().await?;
    let document = Html::parse_document(&text);
    
    let forms = document.select(selectors.login_form());

    let form_node = match forms.last() {
        Some(node) => node,
        None => return Err(Box::new(NotFoundError { value: Cow::Borrowed("Failed to find form!") }))
    };

    let method = form_node.value().attr("method").unwrap().to_owned();
    let action = form_node.value().attr("action").unwrap().to_owned();

    let inputs = form_node
        .select(selectors.login_input())
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
    client: &State<'_, Client>,
    creds: State<'_, Credentials>,
    cookie_jar: State<'_, Arc<CookieStoreMutex>>,
    keyring_entry: State<'_, Entry>,
    login_form_details: LoginFormDetails,
) -> Result<Response, String> {
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
        _ => unreachable!("Invalid method form method: `{}`!", login_form_details.method()),
    }
        .send()
        .await;
    
    

    match response {
        Ok(res) => {
            let status = res.status();
            
            if status.is_success() {
                match cookie_jar.lock() {
                    Ok(inner_jar) => {
                        match bincode::serialize(&inner_jar.iter_unexpired().collect::<Vec<_>>()) {
                            Ok(serialized_value)  => {
                                let base_64_value = base64::encode(serialized_value);
                                keyring_entry.set_password(&base_64_value).unwrap();
                            }
                            Err(e) => eprintln!("Failed to serialize cookies into binary: {}", e),
                        }
                    },
                    Err(e) => eprintln!("Failed to get lock on cookie jar: {}", e),
                }
                Ok(res)
            } else {
                let text = res.text().await;
                Err(format!(
                    "RESERR: Request failed with status code {}!\n\nBody:\n{:?}",
                    status,
                    text.unwrap_or_else(|e| format!("FAILED TO READ BODY TO TEXT: {:?}", e)),
                ))
            }
        },
        Err(e) => Err(format!("REQERR: {}", e.to_string())),
    }
}

pub async fn make_api_request<T>(
    client: &State<'_, Client>,
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

