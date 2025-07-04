use crate::directory_manager::{get_libraries_directory, get_versions_directory};
use crate::structs;
use crate::structs::MinecraftVersion;
use crate::version_manager::load_version_manifest;
use reqwest::Client;
use serde_json::Value;
use std::fs::File;
use std::path::Path;
use std::time::Duration;

pub fn get_current_os() -> String {
    structs::parse_os(sys_info::os_type().expect("Unsupported Operating System"))
}
fn load_downloaded_versions() {
    let dir = get_versions_directory();
    for folder in dir
        .read_dir()
        .unwrap()
        .map(|x| x.unwrap())
        .filter(|x| x.metadata().unwrap().is_dir())
    {
        folder
            .path()
            .read_dir()
            .unwrap()
            .map(|x| x.unwrap())
            .filter(|x| x.path().extension().unwrap() == "json")
            .for_each(|x| {})
    }
}
pub async fn load_versions() -> Vec<MinecraftVersion> {
    let mut versions = Vec::new();
    versions = get_versions_directory()
        .read_dir()
        .unwrap()
        .map(|x| x.unwrap())
        .filter(|x| {
            if x.path().is_file() {
                return false;
            }
            let children_files = x.path().read_dir().unwrap();
            return children_files
                .map(|f| f.unwrap())
                .filter(|f| f.path().is_file() && f.path().extension().unwrap() == "json")
                .count()
                > 0;
        })
        .map(|v| {
            MinecraftVersion::from_folder(
                get_versions_directory().join(v.file_name().to_str().unwrap().to_string()),
            )
            .unwrap()
        })
        .collect();
    if is_connected_to_internet().await {
        let json = load_version_manifest().await;

        let founded_versions = match json {
            None => Vec::new(),
            Some(v) => {
                let versions = v.get("versions").unwrap().as_array().unwrap();
                versions
                    .iter()
                    .filter(|ver| ver.get("type").unwrap() == "release")
                    .map(|ver| {
                        MinecraftVersion::from_id(
                            ver.get("id").unwrap().as_str().unwrap().to_string(),
                        )
                    })
                    .collect()
            }
        };
        versions.extend(founded_versions);
    }
    versions
}

/// Verifies if file exists and is not broken by the expected file size if expected_size is zero it will ignore checking file size
pub fn verify_file_existence(path_str: &String, expected_size: u64) -> bool {
    let path = Path::new(&path_str);
    if !path.exists() {
        false
    } else if expected_size != 0 {
        let file = File::open(path).expect(&("Error ".to_string() + path_str));
        let metadata = file.metadata().unwrap();
        metadata.len() == expected_size
    } else {
        true
    }
}
pub async fn is_connected_to_internet() -> bool {
    let client = Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();

    let req = client
        .get("https://jsonplaceholder.typicode.com/todos/1")
        .send()
        .await;

    match req {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub async fn load_json_url(url: &String) -> Option<Value> {
    let result = reqwest::get(url).await.unwrap();
    let text = result.text().await.unwrap_or(String::new());
    Some(serde_json::from_str(text.as_str()).expect("JSON File isn't well formatted."))
}

pub fn vec_to_string(vec: Vec<String>, separator: String) -> String {
    let mut builder = "".to_string();
    for s in vec {
        builder.push_str(&s);
        builder.push_str(&separator);
    }
    builder.remove(builder.len() - 1);
    builder
}

pub fn parse_library_name_to_path(mavenized_path: String) -> String {
    let parts = mavenized_path.split(":").collect::<Vec<&str>>();
    let group = parts[0].replace(".", "/");
    let artifact_id = parts[1];
    let version = parts[2];
    format!(
        "{}/{group}/{artifact_id}/{version}/{artifact_id}-{version}.jar",
        get_libraries_directory().to_str().unwrap()
    )
}

/// concatenate two vectors without adding repeated indexes
pub fn extend_once<T: PartialEq>(mut vec1: Vec<T>, vec2: Vec<T>) -> Vec<T> {
    for index in vec2 {
        if !vec1.contains(&index) {
            vec1.push(index);
        }
    }
    vec1
}
