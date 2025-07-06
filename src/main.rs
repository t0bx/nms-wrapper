use reqwest::Client;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::{self, Write};
use std::process::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    loop {
        println!("1) Download Mappings");
        println!("2) Download Minecraft-Server.jar");
        println!("3) Download Minecraft-Client.jar");
        println!("4) Execute DataGenerator");
        println!("5) Exit");

        print!("Selection: ");
        io::stdout().flush()?;

        let mut selection = String::new();
        io::stdin().read_line(&mut selection)?;
        let selection = selection.trim();

        if selection == "5" {
            println!("Exiting...");
            break;
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