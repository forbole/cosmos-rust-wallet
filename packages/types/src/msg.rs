//! Transaction Message representation
use prost_types::Any;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct AnyWrapper {
    pub type_url: String,
    pub value: Vec<u8>
}

/// Transaction message wrapper
pub struct Msg(pub(crate) Any);

impl Msg {
    pub fn new(type_url: impl Into<String>, value: impl Into<Vec<u8>>) -> Self {
        Msg(Any {
            type_url: type_url.into(),
            value: value.into(),
        })
    }
}

impl From<Any> for Msg {
    fn from(any: Any) -> Msg {
        Msg(any)
    }
}

impl From<Msg> for Any {
    fn from(msg: Msg) -> Any {
        msg.0
    }
}

impl From<AnyWrapper> for Any {
    fn from(any_wrap: AnyWrapper) -> Any {
        Any{ type_url: any_wrap.type_url, value: any_wrap.value }
    }
}