use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::{self, File};
use tokio::io::AsyncReadExt;
use tokio::sync::RwLock;
use notify::{Watcher, RecursiveMode, Event};
use zip::ZipArchive;
use std::io::Cursor;

use crate::models::comic::{Comic, CoverImage};
use crate::models::error::ComicError;

pub struct ComicService {
    comics_dir: PathBuf,
    comics_cache: Arc<RwLock<HashMap<String, Comic>>>,
    covers_cache: Arc<RwLock<HashMap<String, CoverImage>>>,
}

impl ComicService {
    pub async fn new(comics_dir: PathBuf) -> Result<Self, ComicError> {
        let comics_cache = Arc::new(RwLock::new(HashMap::new()));
        let covers_cache = Arc::new(RwLock::new(HashMap::new()));

        let service = ComicService {
            comics_dir,
            comics_cache,
            covers_cache,
        };

        // Initial scan
        service.scan_directory().await?;

        // Setup file watcher
        service.setup_watcher()?;

        Ok(service)
    }

    async fn scan_directory(&self) -> Result<(), ComicError> {
        let mut entries = fs::read_dir(&self.comics_dir).await?;
        let mut new_comics = HashMap::new();
        let mut new_covers = HashMap::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(extension) = path.extension() {
                if extension.to_string_lossy().to_lowercase() == "cbz" {
                    let comic = self.process_comic_file(&path).await?;
                    let cover = self.extract_cover(&path).await?;

                    let comic_id = comic.id.clone();
                    new_comics.insert(comic_id.clone(), comic);
                    new_covers.insert(comic_id, cover);
                }
            }
        }

        // Update caches
        let mut comics_cache = self.comics_cache.write().await;
        let mut covers_cache = self.covers_cache.write().await;
        *comics_cache = new_comics;
        *covers_cache = new_covers;

        Ok(())
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

        // Create the encoded path first
        let encoded_path = format!("/comics/{}", urlencoding::encode(&file_name));

        Ok(Comic {
            id: file_name.clone(),
            name,
            file_name: file_name,  // file_name is moved here
            path: encoded_path,
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

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let comics_dir = comics_dir.clone();
                let comics_cache = comics_cache.clone();
                let covers_cache = covers_cache.clone();

                tokio::spawn(async move {
                    match event.kind {
                        notify::EventKind::Create(_) |
                        notify::EventKind::Modify(_) |
                        notify::EventKind::Remove(_) => {
                            // Rescan directory on any change
                            let service = ComicService {
                                comics_dir,
                                comics_cache,
                                covers_cache,
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

        watcher.watch(&self.comics_dir, RecursiveMode::NonRecursive)?;

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
        // Try both encoded and decoded versions of the ID
        let cache = self.comics_cache.read().await;
        cache.get(id)
            .or_else(|| {
                urlencoding::decode(id)
                    .ok()
                    .and_then(|decoded| cache.get(decoded.as_ref()))
            })
            .cloned()
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
        // Decode the ID to get the original filename
        let decoded_id = urlencoding::decode(id)
            .map_err(|_| ComicError::InvalidPath)?;

        let comic = self.get_comic(&decoded_id).await
            .ok_or(ComicError::ComicNotFound)?;

        let file_path = self.comics_dir.join(&comic.file_name);
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