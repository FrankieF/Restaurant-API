use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Item {
    pub id: i32,
    pub time_to_prepare: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Table {
    pub id: i32,
    pub items: Vec<Item>,
}

#[derive(Serialize, Deserialize)]
pub struct ItemRequest {
    pub table_id: String,
    pub items: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct StatusMessage {
    pub message: String,
}
