use std::collections::{BTreeMap, HashSet};
use reqwest::Client;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::{self, Write};
use std::process::Command;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    loop {
        println!("1) Download Mappings");
        println!("2) Download Minecraft-Server.jar");
        println!("3) Download Minecraft-Client.jar");
        println!("4) Download Paper-Server.jar");
        println!("5) Compare PacketIds");
        println!("6) Execute DataGenerator");
        println!("7) Exit");

        print!("Selection: ");
        io::stdout().flush()?;

        let mut selection = String::new();
        io::stdin().read_line(&mut selection)?;
        let selection = selection.trim();

        if selection == "7" {
            println!("Exiting...");
            break;
        }

        if selection == "5" {
            print!("What Minecraft Version? (Old) ");
            io::stdout().flush()?;
            let mut protocol_v1_version = String::new();
            io::stdin().read_line(&mut protocol_v1_version)?;
            let protocol_v1_version = protocol_v1_version.trim();

            print!("What Minecraft Version? (New) ");
            io::stdout().flush()?;
            let mut protocol_v2_version = String::new();
            io::stdin().read_line(&mut protocol_v2_version)?;
            let protocol_v2_version = protocol_v2_version.trim();

            let protocol_v1_path = format!("datagenerator/{}/generated/reports/packets.json", protocol_v1_version);
            let protocol_v2_path = format!("datagenerator/{}/generated/reports/packets.json", protocol_v2_version);

            let json_v1 = match fs::read_to_string(&protocol_v1_path) {
                Ok(data) => data,
                Err(_) => {
                    println!("Could not read file: {}", protocol_v1_path);
                    continue;
                }
            };

            let json_v2 = match fs::read_to_string(&protocol_v2_path) {
                Ok(data) => data,
                Err(_) => {
                    println!("Could not read file: {}", protocol_v2_path);
                    continue;
                }
            };

            let filter = ask_for_filter();

            let v1: Value = serde_json::from_str(&json_v1)?;
            let v2: Value = serde_json::from_str(&json_v2)?;

            let map_v1 = extract_protocol_ids(&v1, vec![]);
            let map_v2 = extract_protocol_ids(&v2, vec![]);

            let keys_v1: HashSet<_> = map_v1.keys().collect();
            let keys_v2: HashSet<_> = map_v2.keys().collect();
            let all_keys: HashSet<_> = keys_v1.union(&keys_v2).collect();

            println!();
            println!("=== Differences from {} → {} ===", protocol_v1_version, protocol_v2_version);
            println!();

            let mut differences_found = false;

            for key in all_keys {
                if let Some(ref filter_set) = filter {
                    if !filter_set.iter().any(|f| key.to_lowercase().contains(f)) {
                        continue;
                    }
                }

                let id_v1 = map_v1.get(*key);
                let id_v2 = map_v2.get(*key);

                match (id_v1, id_v2) {
                    (Some(old_id), Some(new_id)) => {
                        if old_id != new_id {
                            println!(
                                "{}: {} (0x{:X}) → {} (0x{:X})",
                                key, old_id, old_id, new_id, new_id
                            );
                            differences_found = true;
                        }
                    }
                    (None, Some(new_id)) => {
                        println!(
                            "{}: new → {} (0x{:X})",
                            key, new_id, new_id
                        );
                        differences_found = true;
                    }
                    (Some(old_id), None) => {
                        println!(
                            "{}: removed (was {} / 0x{:X})",
                            key, old_id, old_id
                        );
                        differences_found = true;
                    }
                    (None, None) => {}
                }
            }

            if !differences_found {
                println!("No differences found.");
            }

            println!();
            continue;
        }

        print!("What Minecraft Version? ");
        io::stdout().flush()?;

        let mut version_id = String::new();
        io::stdin().read_line(&mut version_id)?;
        let version_id = version_id.trim();

        let manifest_url = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
        let manifest: Manifest = client.get(manifest_url).send().await?.json().await?;

        let version = match manifest.versions.iter().find(|v| v.id == version_id) {
            Some(v) => v,
            None => {
                println!("Version '{}' not found.", version_id);
                continue;
            }
        };

        println!("Version found: {}", version.url);

        let version_data: VersionData = client.get(&version.url).send().await?.json().await?;

        match selection {
            "1" => {
                let client_url = &version_data.downloads.client_mappings.url;
                let server_url = &version_data.downloads.server_mappings.url;

                let mappings_dir = format!("mappings/{}/", version_id);
                fs::create_dir_all(&mappings_dir)?;

                let client_path = format!("{}client-mappings.txt", mappings_dir);
                let server_path = format!("{}server-mappings.txt", mappings_dir);

                download_file(client_url, &client_path, &client).await?;
                download_file(server_url, &server_path, &client).await?;

                println!("Mappings successfully downloaded.");
            }
            "2" => {
                let server_jar_url = &version_data.downloads.server.url;

                let server_dir = format!("server-versions/{}/", version_id);
                fs::create_dir_all(&server_dir)?;

                let server_path = format!("{}server.jar", server_dir);

                download_file(server_jar_url, &server_path, &client).await?;

                println!("Server.jar downloaded into {}", server_path);
            }
            "3" => {
                let client_jar_url = &version_data.downloads.client.url;

                let client_dir = format!("client-versions/{}/", version_id);
                fs::create_dir_all(&client_dir)?;

                let client_path = format!("{}client.jar", client_dir);

                download_file(client_jar_url, &client_path, &client).await?;

                println!("Client.jar downloaded into {}", client_path);
            }
            "4" => {
                let paper_manifest_url = "https://gist.githubusercontent.com/osipxd/6119732e30059241c2192c4a8d2218d9/raw/471f25cc5c9ca724e6493ed5e266770d7d307621/paper-versions.json";
                
                let paper_manifest: PaperVersions = client
                    .get(paper_manifest_url)
                    .send()
                    .await?
                    .json()
                    .await?;
                
                if let Some(paper_url) = paper_manifest.versions.get(version_id) {
                    let paper_dir = format!("paper-versions/{}/", version_id);
                    fs::create_dir_all(&paper_dir)?;
                    
                    let paper_path = format!("{}paper.jar", paper_dir);
                    
                    download_file(paper_url, &paper_path, &client).await?;
                    
                    println!("Paper.jar downloaded into {}", paper_path);
                } else {
                    println!("Paper.jar not found for this version '{}'.", version_id);  
                }
            }
            "6" => {
                let server_path = match fs::canonicalize(format!("server-versions/{}/server.jar", version_id)) {
                    Ok(path) => path,
                    Err(_) => {
                        println!("Server.jar not found! Download it first with Option 2.");
                        continue;
                    }
                };

                let data_dir = format!("datagenerator/{}/", version_id);
                fs::create_dir_all(&data_dir)?;

                let status = Command::new("java")
                    .arg("-DbundlerMainClass=net.minecraft.data.Main")
                    .arg("-jar")
                    .arg(&server_path)
                    .arg("--all")
                    .current_dir(&data_dir)
                    .status()?;

                if status.success() {
                    println!("Data Generator successfully executed.");
                } else {
                    println!("Data Generator failed.");
                }
            }
            _ => {
                println!("Unknown Selection.");
            }
        }
    }

    Ok(())
}

async fn download_file(
    url: &str,
    path: &str,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Downloading: {}", url);

    let mut response = client.get(url).send().await?;
    let bytes = response.bytes().await?;
    let mut file = File::create(path)?;

    io::copy(&mut bytes.as_ref(), &mut file)?;

    println!("Saved: {}", path);
    Ok(())
}

#[derive(Deserialize)]
struct PaperVersions {
    latest: String,
    versions: std::collections::HashMap<String, String>,
}

#[derive(Deserialize)]
struct Manifest {
    versions: Vec<Version>,
}

#[derive(Deserialize)]
struct Version {
    id: String,
    url: String,
}

#[derive(Deserialize)]
struct VersionData {
    downloads: Downloads,
}

#[derive(Deserialize)]
struct Downloads {
    #[serde(rename = "client_mappings")]
    client_mappings: DownloadUrl,
    #[serde(rename = "server_mappings")]
    server_mappings: DownloadUrl,
    server: DownloadUrl,
    client: DownloadUrl,
}

#[derive(Deserialize)]
struct DownloadUrl {
    url: String,
}

fn ask_for_filter() -> Option<HashSet<String>> {
    print!("Do you want to compare specific packets only? (Yes/No): ");
    io::stdout().flush().unwrap();

    let mut answer = String::new();
    io::stdin().read_line(&mut answer).unwrap();
    let answer = answer.trim().to_lowercase();

    if answer == "yes" || answer == "y" {
        match fs::read_to_string("filters.txt") {
            Ok(content) => {
                let filter_set = content
                    .split(',')
                    .map(|s| s.trim().to_lowercase())
                    .filter(|s| !s.is_empty())
                    .collect::<HashSet<_>>();
                println!("Loaded {} filter(s)", filter_set.len());
                Some(filter_set)
            }
            Err(_) => {
                println!("Could not read 'filters.txt'. Showing all differences instead.");
                None
            }
        }
    } else {
        None
    }
}

fn extract_protocol_ids(value: &Value, path: Vec<String>) -> BTreeMap<String, u32> {
    let mut map = BTreeMap::new();

    match value {
        Value::Object(obj) => {
            if let Some(Value::Number(n)) = obj.get("protocol_id") {
                if let Some(id) = n.as_u64() {
                    let path_str = path.join(".");
                    map.insert(path_str, id as u32);
                }
            }

            for (k, v) in obj {
                let mut new_path = path.clone();
                new_path.push(k.clone());
                let inner_map = extract_protocol_ids(v, new_path);
                map.extend(inner_map);
            }
        }
        _ => {
        }
    }

    map
}