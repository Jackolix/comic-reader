use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct Comic {
    pub id: String,
    pub name: String,
    pub file_name: String,
    pub path: String,
    pub folder_path: Vec<String>,
    pub series: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CoverImage {
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Folder {
    pub name: String,
    pub path: Vec<String>,
    pub comics: Vec<Comic>,
    pub subfolders: Vec<Folder>,
}

impl Comic {
    pub fn from_path(base_dir: &PathBuf, full_path: &PathBuf) -> Option<Self> {

        let file_name = full_path.file_name()?.to_string_lossy().into_owned();
        let name = full_path.file_stem()?.to_string_lossy().into_owned();

        // Calculate relative path from base_dir
        let relative_path = full_path.strip_prefix(base_dir).ok()?;
        let parent_path = relative_path.parent();


        // Create folder path and series name
        let (folder_path, series) = if let Some(parent) = parent_path {
            if parent.as_os_str().is_empty() {
                (vec![], None)
            } else {
                let folder_path: Vec<String> = parent
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect();
                let series = folder_path.last().cloned();
                (folder_path, series)
            }
        } else {
            (vec![], None)
        };


        // Create the encoded path
        let encoded_path = format!("/comics/{}", urlencoding::encode(&file_name));

        let comic = Comic {
            id: file_name.clone(),
            name,
            file_name,
            path: encoded_path,
            folder_path,
            series,
        };

        Some(comic)
    }


    pub fn matches_search(&self, query: &str) -> bool {
        let query = query.to_lowercase();

        // Check if query matches name
        if self.name.to_lowercase().contains(&query) {
            return true;
        }

        // Check if query matches series/folder
        if let Some(series) = &self.series {
            if series.to_lowercase().contains(&query) {
                return true;
            }
        }

        // Check if query matches any folder in path
        self.folder_path.iter().any(|folder|
            folder.to_lowercase().contains(&query)
        )
    }
}