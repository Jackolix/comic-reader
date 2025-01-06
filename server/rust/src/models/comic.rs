use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Comic {
    pub id: String,
    pub name: String,
    pub file_name: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct CoverImage {
    pub data: Vec<u8>,
}