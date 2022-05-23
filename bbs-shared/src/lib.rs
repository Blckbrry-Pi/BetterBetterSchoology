pub mod reducer_actions;
pub mod state;

pub mod data;

use serde::{Serialize, Deserialize};

pub use state::PageState;
pub use reducer_actions::{ StateUpdateAction, DataUpdateAction };

pub use data::FrontendData;


#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct ClassID(pub u64);

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct ClassItemID(pub u64);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DueDate();



#[macro_export]
macro_rules! add_base64 {
    ($struct_name: ty) => {

        impl $struct_name {
            pub fn from_base64(base64: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
                let data = base64::decode(base64)?;
                let instance = bincode::deserialize::<Self>(&data)?;
                Ok(instance)
            }

            pub fn to_base64(&self) -> Result<String, Box<dyn std::error::Error>> {
                let data = bincode::serialize(self)?;
                
                Ok(base64::encode(data))
            }
        }

        impl TryFrom<&[u8]> for $struct_name
            {
            type Error = Box<dyn std::error::Error>;

            fn try_from(input: &[u8]) -> Result<Self, Self::Error> {
                Self::from_base64(input)
            }
        }

        impl TryInto<String> for $struct_name {
            type Error = Box<dyn std::error::Error>;
            fn try_into(self) -> Result<String, Self::Error> {
                self.to_base64()
            }
        }
    };
}