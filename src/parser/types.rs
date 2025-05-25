use serde::{Deserialize, Serialize};

use crate::relic::Slot;

#[derive(Debug, Serialize, Deserialize)]
pub struct RelicSetStub {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RelicStub {
    pub set_id: String,
    #[serde(rename = "type")]
    pub slot: String,
    pub name: String,
}
