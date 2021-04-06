use cosmos_sdk_proto::cosmos::tx::v1beta1::Fee;
use prost_types::Any;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AnyWrapper {
    pub type_url: String,
    pub value: Vec<u8>,
}

impl From<AnyWrapper> for Any {
    fn from(any_wrap: AnyWrapper) -> Any {
        Any {
            type_url: any_wrap.type_url,
            value: any_wrap.value,
        }
    }
}

impl From<AnyWrapper> for Fee {
    fn from(any_wrap: AnyWrapper) -> Fee {
        prost::Message::decode(any_wrap.value.as_slice()).unwrap()
    }
}
