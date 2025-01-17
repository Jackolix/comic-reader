use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;
use tokio::fs::{self, File};
use tokio::io::AsyncReadExt;
use tokio::sync::RwLock;
use notify::{Watcher, RecursiveMode, Event};
use zip::ZipArchive;
use std::io::Cursor;

use crate::models::comic::{Comic, CoverImage, Folder};
use crate::models::error::ComicError;

pub struct ComicService {
    comics_dir: PathBuf,
    comics_cache: Arc<RwLock<HashMap<String, Comic>>>,
    covers_cache: Arc<RwLock<HashMap<String, CoverImage>>>,
    folder_structure: Arc<RwLock<Folder>>,
}

impl ComicService {
    pub async fn new(comics_dir: PathBuf) -> Result<Self, ComicError> {
        let comics_cache = Arc::new(RwLock::new(HashMap::new()));
        let covers_cache = Arc::new(RwLock::new(HashMap::new()));
        let folder_structure = Arc::new(RwLock::new(Folder {
            name: "root".to_string(),
            path: vec![],
            comics: vec![],
            subfolders: vec![],
        }));

        let service = ComicService {
            comics_dir,
            comics_cache,
            covers_cache,
            folder_structure,
        };

        // Initial scan
        service.scan_directory().await?;

        // Setup file watcher with recursive mode
        service.setup_watcher()?;

        Ok(service)
    }

    async fn scan_directory(&self) -> Result<(), ComicError> {
        println!("\nStarting full directory scan");
        let mut new_comics = HashMap::new();
        let mut new_covers = HashMap::new();
        let mut root_folder = Folder {
            name: "root".to_string(),
            path: vec![],
            comics: vec![],
            subfolders: vec![],
        };

        // Recursive directory scanning
        self.scan_directory_recursive(&self.comics_dir, &mut new_comics, &mut new_covers, &mut root_folder).await?;

        println!("\nScan complete:");
        println!("Total comics found: {}", new_comics.len());
        println!("Root comics: {}", root_folder.comics.len());
        println!("Root subfolders: {}", root_folder.subfolders.len());

        // Update caches
        let mut comics_cache = self.comics_cache.write().await;
        let mut covers_cache = self.covers_cache.write().await;
        let mut folder_structure = self.folder_structure.write().await;

        *comics_cache = new_comics;
        *covers_cache = new_covers;
        *folder_structure = root_folder;

        Ok(())
    }

    fn scan_directory_recursive<'a>(
        &'a self,
        dir: &'a Path,
        comics: &'a mut HashMap<String, Comic>,
        covers: &'a mut HashMap<String, CoverImage>,
        current_folder: &'a mut Folder,
    ) -> Pin<Box<dyn Future<Output = Result<(), ComicError>> + Send + 'a>> {
        Box::pin(async move {

            let mut entries = fs::read_dir(dir).await?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();

                if path.is_dir() {
                    let folder_name = path.file_name()
                        .ok_or_else(|| ComicError::InvalidPath)?
                        .to_string_lossy()
                        .into_owned();

                    let mut subfolder = Folder {
                        name: folder_name.clone(),
                        path: current_folder.path.iter()
                            .cloned()
                            .chain(std::iter::once(current_folder.name.clone()))
                            .filter(|name| name != "root")
                            .collect(),
                        comics: vec![],
                        subfolders: vec![],
                    };

                    self.scan_directory_recursive(&path, comics, covers, &mut subfolder).await?;

                    if !subfolder.comics.is_empty() || !subfolder.subfolders.is_empty() {
                        current_folder.subfolders.push(subfolder);
                    }
                } else if let Some(extension) = path.extension() {
                    if extension.to_string_lossy().to_lowercase() == "cbz" {
                        if let Some(comic) = Comic::from_path(&self.comics_dir, &path) {
                            if let Ok(cover) = self.extract_cover(&path).await {
                                covers.insert(comic.id.clone(), cover);
                                comics.insert(comic.id.clone(), comic.clone());
                                current_folder.comics.push(comic);
                            }
                        }
                    }
                }
            }

            println!("Finished scanning directory: {}", dir.display());
            println!("Current folder {} now has {} comics and {} subfolders",
                     current_folder.name, current_folder.comics.len(), current_folder.subfolders.len());

            Ok(())
        })
    }

    pub async fn search_comics(&self, query: &str) -> Vec<Comic> {
        let comics_cache = self.comics_cache.read().await;
        comics_cache.values()
            .filter(|comic| comic.matches_search(query))
            .cloned()
            .collect()
    }

    pub async fn get_folder_structure(&self) -> Folder {
        self.folder_structure.read().await.clone()
    }

    async fn process_comic_file(&self, path: &Path) -> Result<Comic, ComicError> {
        let file_name = path.file_name()
            .ok_or_else(|| ComicError::InvalidPath)?
            .to_string_lossy()
            .into_owned();
        let name = path.file_stem()
            .ok_or_else(|| ComicError::InvalidPath)?
            .to_string_lossy()
            .into_owned();

        // Calculate relative path from base_dir
        let relative_path = path.strip_prefix(&self.comics_dir)
            .map_err(|_| ComicError::InvalidPath)?;
        let parent_path = relative_path.parent();

        // Create folder path
        let folder_path: Vec<String> = if let Some(parent) = parent_path {
            parent.components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect()
        } else {
            vec![]
        };

        // Get series name (immediate parent folder)
        let series = folder_path.last().cloned();

        // Create the encoded path
        let encoded_path = format!("/comics/{}", urlencoding::encode(&file_name));

        Ok(Comic {
            id: file_name.clone(),
            name,
            file_name,
            path: encoded_path,
            folder_path,
            series,
        })
    }

    async fn extract_cover(&self, path: &Path) -> Result<CoverImage, ComicError> {
        let mut file = File::open(path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        let mut archive = ZipArchive::new(Cursor::new(buffer))?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_lowercase();

            if name.ends_with(".jpg") || name.ends_with(".jpeg") ||
                name.ends_with(".png") || name.ends_with(".gif") {
                let mut data = Vec::new();
                std::io::copy(&mut file, &mut data)?;
                return Ok(CoverImage { data });
            }
        }

        Err(ComicError::NoCoverFound)
    }

    fn setup_watcher(&self) -> Result<(), ComicError> {
        let comics_dir = self.comics_dir.clone();
        let comics_cache = self.comics_cache.clone();
        let covers_cache = self.covers_cache.clone();
        let folder_structure = self.folder_structure.clone();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let comics_dir = comics_dir.clone();
                let comics_cache = comics_cache.clone();
                let covers_cache = covers_cache.clone();
                let folder_structure = folder_structure.clone();

                // Create a runtime for the async operations
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.spawn(async move {
                    match event.kind {
                        notify::EventKind::Create(_) |
                        notify::EventKind::Modify(_) |
                        notify::EventKind::Remove(_) => {
                            let service = ComicService {
                                comics_dir,
                                comics_cache,
                                covers_cache,
                                folder_structure,
                            };
                            if let Err(e) = service.scan_directory().await {
                                eprintln!("Error rescanning directory: {}", e);
                            }
                        },
                        _ => (),
                    }
                });
            }
        })?;

        watcher.watch(&self.comics_dir, RecursiveMode::Recursive)?;

        // Keep watcher alive by storing it in a thread-local or global state
        std::thread::spawn(move || {
            std::mem::forget(watcher);
        });

        Ok(())
    }

    pub async fn get_all_comics(&self) -> Vec<Comic> {
        self.comics_cache.read()
            .await
            .values()
            .cloned()
            .collect()
    }

    pub async fn get_comic(&self, id: &str) -> Option<Comic> {
        println!("get_comic called with id: {}", id);
        
        let actual_id = if id.contains('/') {
            let filename = id.split('/').last()?;
            println!("Split filename: {}", filename);
            filename
        } else {
            println!("Using id as-is: {}", id);
            id
        };
    
        let cache = self.comics_cache.read().await;
        println!("Cache keys: {:?}", cache.keys().collect::<Vec<_>>());
        
        let result = cache.get(actual_id)
            .or_else(|| {
                urlencoding::decode(actual_id)
                    .ok()
                    .and_then(|decoded| cache.get(decoded.as_ref()))
            })
            .cloned();
            
        println!("Found in cache: {}", result.is_some());
        result
    }

    pub async fn get_cover(&self, id: &str) -> Option<CoverImage> {
        // Try both encoded and decoded versions of the ID
        let cache = self.covers_cache.read().await;
        cache.get(id)
            .or_else(|| {
                urlencoding::decode(id)
                    .ok()
                    .and_then(|decoded| cache.get(decoded.as_ref()))
            })
            .cloned()
    }

    pub async fn get_comic_data(&self, id: &str) -> Result<Vec<u8>, ComicError> {
        // Extract the folder path and filename
        let (folder_path, filename) = if id.contains('/') {
            let parts: Vec<&str> = id.rsplitn(2, '/').collect();
            if parts.len() == 2 {
                (Some(parts[1]), parts[0])
            } else {
                (None, id)
            }
        } else {
            (None, id)
        };
    
        // Decode the filename to get the original filename
        let decoded_filename = urlencoding::decode(filename)
            .map_err(|_| ComicError::InvalidPath)?;
        
        let comic = self.get_comic(&decoded_filename).await
            .ok_or(ComicError::ComicNotFound)?;
        
        // Construct the full path including any folders
        let mut file_path = self.comics_dir.clone();
        
        // Add folder path if it exists either from the URL or from the comic metadata
        if let Some(folder) = folder_path {
            file_path.push(folder);
        } else if !comic.folder_path.is_empty() {
            for folder in &comic.folder_path {
                file_path.push(folder);
            }
        }
        
        // Add the actual file name
        file_path.push(&comic.file_name);
    
        println!("Attempting to read comic from: {}", file_path.display());
    
        let mut file = File::open(&file_path).await
            .map_err(|e| {
                println!("Failed to open file: {}", e);
                ComicError::IoError(e)
            })?;
    
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;
    
        Ok(buffer)
    }
}