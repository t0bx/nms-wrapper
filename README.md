# nms-wrapper

A Rust CLI tool to download Minecraft server/client jars and mappings, and run the Minecraft Data Generator for any official version.

## Features
- Download official Minecraft server and client jars for any version
- Download official Mojang mappings (client/server) for any version
- Run the Minecraft Data Generator using the downloaded server jar
- Simple interactive CLI

## Usage

1. **Build the project:**
   ```sh
   cargo build --release
   ```
2. **Run the CLI:**
   ```sh
   cargo run --release
   ```
3. **Follow the prompts:**
   - Choose an option (download mappings, server/client jar, run data generator, or exit)
   - Enter the desired Minecraft version (e.g., `1.20.4`)

### Example
```
1) Download Mappings
2) Download Minecraft-Server.jar
3) Download Minecraft-Client.jar
4) Execute DataGenerator
5) Exit
Selection: 1
What Minecraft Version? 1.20.4
```

## Output
- Downloads are saved in versioned subdirectories:
  - `mappings/<version>/`
  - `server-versions/<version>/`
  - `client-versions/<version>/`
  - `datagenerator/<version>/`

## Requirements
- Rust (edition 2024)
- Java (for Data Generator)

## Dependencies
- [tokio](https://crates.io/crates/tokio)
- [reqwest](https://crates.io/crates/reqwest)
- [serde](https://crates.io/crates/serde)
- [serde_json](https://crates.io/crates/serde_json)