use crossterm::event::{self, Event, KeyCode};
use reqwest::header::{HeaderMap, HeaderValue, ACCESS_CONTROL_ALLOW_ORIGIN};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{Read, Write},
    sync::Arc,
};
use tokio::{
    sync::{mpsc, Mutex},
    time::{interval, Duration},
};

use base64::prelude::*;
use reqwest::Client;
use serde_json::Value;

#[derive(Clone)]
struct SpotifyToken {
    access_token: String,
    _expires_in: u64,
    _scope: String,
    token_type: String,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
struct Shortcut {
    is: String, // shortcut or folder
    name: String,
    command: String,
    icon: String,
    shortcuts: Option<Vec<Shortcut>>,
}

async fn handle_connection(
    mut stream: std::net::TcpStream,
    token: &Arc<Mutex<SpotifyToken>>,
    shortcuts: &Arc<Mutex<Vec<Shortcut>>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer[..]);
    let request_body = request.split("\r\n\r\n").collect::<Vec<&str>>();
    let request_body = if request_body.len() > 1 {
        request_body[1]
    } else {
        return Ok(());
    };

    let commands = request_body.split_whitespace().collect::<Vec<&str>>();
    let cmd = commands[0].trim_matches(char::from(0));

    println!(
        "Received command: {} {}",
        cmd,
        if commands.len() > 1 { commands[1] } else { "" }
    );

    let token = token.lock().await.clone();
    let shortcuts = shortcuts.lock().await.clone();

    let response = match cmd {
        "SP_DEVICES" => {
            let client = Client::new();
            let response = client
                .get("https://api.spotify.com/v1/me/player/devices")
                .header(
                    "Authorization",
                    format!("{} {}", token.token_type, token.access_token),
                )
                .send()
                .await?
                .text()
                .await?;
            response
        }
        "SP_TRANSFER" => {
            // SP_TRANSFER <device_id>
            let client = Client::new();
            let device_id = commands[1].trim_matches(char::from(0));
            let response = client
                .put(format!("https://api.spotify.com/v1/me/player"))
                .header(
                    "Authorization",
                    format!("{} {}", token.token_type, token.access_token),
                )
                .header("Content-Type", "application/json")
                .body(format!(
                    r#"{{
                        "device_ids": ["{device_id}"],
                        "play": true
                    }}"#,
                    device_id = device_id
                ))
                .send()
                .await?
                .text()
                .await?;
            response
        }
        "SP_CURRENT_TRACK" => {
            let client = Client::new();
            let response = client
                .get("https://api.spotify.com/v1/me/player/currently-playing")
                .header(
                    "Authorization",
                    format!("{} {}", token.token_type, token.access_token),
                )
                .send()
                .await?
                .text()
                .await?;
            response
        }
        "SP_PLAY" => {
            let client = Client::new();
            let response = client
                .put("https://api.spotify.com/v1/me/player/play")
                .header(
                    "Authorization",
                    format!("{} {}", token.token_type, token.access_token),
                )
                .header("Content-Length", "0")
                .send()
                .await?
                .text()
                .await?;
            response
        }
        "SP_PAUSE" => {
            let client = Client::new();
            let response = client
                .put("https://api.spotify.com/v1/me/player/pause")
                .header(
                    "Authorization",
                    format!("{} {}", token.token_type, token.access_token),
                )
                .header("Content-Length", "0")
                .send()
                .await?
                .text()
                .await?;
            response
        }
        "SP_NEXT" => {
            let client = Client::new();
            let response = client
                .post("https://api.spotify.com/v1/me/player/next")
                .header(
                    "Authorization",
                    format!("{} {}", token.token_type, token.access_token),
                )
                .header("Content-Length", "0")
                .send()
                .await?
                .text()
                .await?;
            response
        }
        "SP_PREV" => {
            let client = Client::new();
            let response = client
                .post("https://api.spotify.com/v1/me/player/previous")
                .header(
                    "Authorization",
                    format!("{} {}", token.token_type, token.access_token),
                )
                .header("Content-Length", "0")
                .send()
                .await?
                .text()
                .await?;
            response
        }
        "SP_SEEK" => {
            // SP_SEEK <position>
            let client = Client::new();
            let pos = commands[1].trim_matches(char::from(0));
            let response = client
                .put(format!(
                    "https://api.spotify.com/v1/me/player/seek?position_ms={pos}"
                ))
                .header(
                    "Authorization",
                    format!("{} {}", token.token_type, token.access_token),
                )
                .header("Content-Type", "application/json")
                .header("Content-Length", "0")
                .send()
                .await?
                .text()
                .await?;
            response
        }
        "SC_GET" => {
            let mut response = String::from("[");
            for (i, shortcut) in shortcuts.iter().enumerate() {
                response.push_str(&serde_json::to_string(shortcut).unwrap());
                if i < shortcuts.len() - 1 {
                    response.push_str(",");
                }
            }
            response.push_str("]");
            response
        }
        "SC_RUN" => {
            // SC_RUN <name>
            let joined_commands = commands[1..].join(" ");
            let name = joined_commands.trim_matches(char::from(0));
            if name.contains('/') {
                let folder_name = name.split('/').collect::<Vec<&str>>()[0];
                let shortcut_name = name.split('/').collect::<Vec<&str>>()[1];
                let folder = shortcuts
                    .iter()
                    .find(|s: &&Shortcut| s.name == folder_name)
                    .unwrap();
                let shortcut = folder
                    .shortcuts
                    .as_ref()
                    .unwrap()
                    .iter()
                    .find(|s: &&Shortcut| s.name == shortcut_name)
                    .unwrap();
                let response = std::process::Command::new("cmd")
                    .arg("/C")
                    .arg(shortcut.command.clone())
                    .output()
                    .unwrap();
                format!(
                    r#"{{
                        "stdout": "{stdout}",
                        "stderr": "{stderr}",
                        "status": "{status}"
                    }}"#,
                    stdout = String::from_utf8_lossy(&response.stdout),
                    stderr = String::from_utf8_lossy(&response.stderr),
                    status = response.status
                )
            } else {
                let shortcut = shortcuts
                    .iter()
                    .find(|s: &&Shortcut| s.name == name)
                    .unwrap();
                let response = std::process::Command::new("cmd")
                    .arg("/C")
                    .arg(shortcut.command.clone())
                    .output()
                    .unwrap();
                format!(
                    r#"{{
                        "stdout": "{stdout}",
                        "stderr": "{stderr}",
                        "status": "{status}"
                    }}"#,
                    stdout = String::from_utf8_lossy(&response.stdout),
                    stderr = String::from_utf8_lossy(&response.stderr),
                    status = response.status
                )
            }
        }
        _ => format!("{{error: \"Unknown command\", cmd: \"{cmd}\"}}"),
    };

    let mut headers = HeaderMap::new();
    headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
        response.len(),
        response
    );

    stream.write(response.as_bytes()).unwrap();

    Ok(())
}

async fn start_server(
    ip: &str,
    port: u16,
    running: &Arc<Mutex<bool>>,
    token: Arc<Mutex<SpotifyToken>>,
    shortcuts: Arc<Mutex<Vec<Shortcut>>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("{}:{}", ip, port);
    let listener = std::net::TcpListener::bind(&addr).unwrap();

    for stream in listener.incoming() {
        if !*running.lock().await {
            break;
        }

        let stream = stream.unwrap();
        handle_connection(stream, &token, &shortcuts).await.unwrap();
    }

    Ok(())
}

async fn get_spotify_token(
    spotify_id: &str,
    spotify_secret: &str,
    spotify_token: &str,
) -> Result<SpotifyToken, Box<dyn std::error::Error + Send + Sync>> {
    let auth_header = format!(
        "Basic {}",
        BASE64_STANDARD.encode(format!("{}:{}", spotify_id, spotify_secret))
    );

    let mut params = HashMap::new();
    params.insert("grant_type", "refresh_token");
    params.insert("refresh_token", spotify_token);

    let client = Client::new();
    let response = client
        .post("https://accounts.spotify.com/api/token")
        .header("Authorization", auth_header)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await?;

    let response = response.text().await?;
    let response: Value = serde_json::from_str(&response)?;

    let token = SpotifyToken {
        access_token: response["access_token"].as_str().unwrap().to_string(),
        _expires_in: response["expires_in"].as_u64().unwrap(),
        _scope: response["scope"].as_str().unwrap().to_string(),
        token_type: response["token_type"].as_str().unwrap().to_string(),
    };

    Ok(token)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // read config.json file
    let config = std::fs::read_to_string("config.json").unwrap();
    let config: Value = serde_json::from_str(&config).unwrap();

    let spotify_id = config["spotify_id"].as_str().unwrap().to_string();
    let spotify_secret = config["spotify_secret"].as_str().unwrap().to_string();
    let spotify_token = config["spotify_token"].as_str().unwrap().to_string();

    let shortcuts_config = config["shortcuts"].as_array().unwrap();
    let mut new_shortcuts = Vec::new();
    for shortcut in shortcuts_config {
        let shortcut: Shortcut = serde_json::from_value(shortcut.clone()).unwrap();
        new_shortcuts.push(shortcut);
    }
    let shortcuts = Arc::new(Mutex::new(new_shortcuts));

    let token = Arc::new(Mutex::new(
        get_spotify_token(&spotify_id, &spotify_secret, &spotify_token).await?,
    ));

    let token_clone = Arc::clone(&token);
    let token_for_server = Arc::clone(&token);
    let spotify_id_clone = spotify_id.clone();
    let spotify_secret_clone = spotify_secret.clone();
    let spotify_token_clone = spotify_token.clone();

    // refresh token every 30 minutes
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(60 * 30));
        loop {
            interval.tick().await;
            let new_token = get_spotify_token(
                &spotify_id_clone,
                &spotify_secret_clone,
                &spotify_token_clone,
            )
            .await;
            match new_token {
                Ok(token_data) => {
                    let mut token_lock = token_clone.lock().await;
                    *token_lock = token_data;
                    println!("Token refreshed");
                }
                Err(e) => {
                    eprintln!("Failed to refresh token: {}", e);
                }
            }
        }
    });

    // Create a channel to receive key presses
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Spawn a task to listen for key presses
    tokio::spawn(async move {
        loop {
            if event::poll(Duration::from_millis(100)).unwrap() {
                if let Event::Key(key_event) = event::read().unwrap() {
                    if key_event.kind != crossterm::event::KeyEventKind::Press {
                        continue;
                    }

                    match key_event.code {
                        KeyCode::Char('q') => {
                            println!("Quit key pressed");
                            let _ = tx.send('q');
                            break;
                        }
                        KeyCode::Char('r') => {
                            println!("Reload key pressed");
                            let _ = tx.send('r');
                        }
                        KeyCode::Char(ch) => {
                            println!("Key pressed: {}", ch);
                            let _ = tx.send(ch);
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    let running = Arc::new(Mutex::new(true));
    let running_clone = Arc::clone(&running);

    let shortcuts_clone = Arc::clone(&shortcuts);
    let mut server_task = tokio::spawn(async move {
        start_server(
            "0.0.0.0",
            8008,
            &running_clone,
            token_for_server,
            shortcuts_clone,
        )
        .await
        .unwrap();
    });

    loop {
        tokio::select! {
            Some(key) = rx.recv() => {
                if key == 'q' {
                    let mut running_lock = running.lock().await;
                    *running_lock = false;
                    break;
                } else if key == 'r' {
                    // Reload shortcuts
                    let config = std::fs::read_to_string("config.json").unwrap();
                    let config: Value = serde_json::from_str(&config).unwrap();

                    let new_shortcuts_config = config["shortcuts"].as_array().unwrap();
                    let mut new_shortcuts = Vec::new();
                    for shortcut in new_shortcuts_config {
                        let shortcut: Shortcut = serde_json::from_value(shortcut.clone()).unwrap();
                        new_shortcuts.push(shortcut);
                    }
                    let mut shortcuts_lock = shortcuts.lock().await;
                    *shortcuts_lock = new_shortcuts;

                    // Reload token
                    let spotify_id = config["spotify_id"].as_str().unwrap().to_string();
                    let spotify_secret = config["spotify_secret"].as_str().unwrap().to_string();
                    let spotify_token = config["spotify_token"].as_str().unwrap().to_string();
                    let new_token = get_spotify_token(&spotify_id, &spotify_secret, &spotify_token).await;
                    match new_token {
                        Ok(token_data) => {
                            let mut token_lock = token.lock().await;
                            *token_lock = token_data;
                            println!("Token refreshed");
                        }
                        Err(e) => {
                            eprintln!("Failed to refresh token: {}", e);
                        }
                    }
                }
            }
            _ = &mut server_task => {
                // Server task completed
                break;
            }
        }
    }

    Ok(())
}
