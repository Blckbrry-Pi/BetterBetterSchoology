use bincode::ErrorKind;
use serde::{Serialize, Deserialize};
use std::{error::Error, fmt::Display};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CredSetError;

impl Into<String> for CredSetError {
    fn into(self) -> String {
        base64::encode(bincode::serialize(&self).unwrap())
    }
}

impl Display for CredSetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Failed to set credentials!")
    }
}

impl Error for CredSetError {}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoginError {
    SerializationError,
    FindFormError,
    InvalidCredsError,
    RequestError,
    LaterRequestError,
    DecodeError,
    JsonError,
}

impl Into<String> for LoginError {
    fn into(self) -> String {
        base64::encode(bincode::serialize(&self).unwrap())
    }
}
impl TryFrom<String> for LoginError {
    type Error = Box<ErrorKind>;
    fn try_from(string: String) -> Result<Self, Box<ErrorKind>> {
        bincode::deserialize(&base64::decode(string).unwrap())
    }
}

// TODO: Implement later!
// impl Display for CredSetError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.write_str("Failed to set credentials!")
//     }
// }

// impl Error for CredSetError {}