use tauri::State;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::fs::File;
use tokio::time::{timeout, Duration};
use tokio::task;
use tokio::sync::Mutex;
use tokio::process::Command;
use std::net::{IpAddr, Ipv4Addr};
use serde::Serialize;
use hostname::get;
use futures::StreamExt;
use local_ip_address::local_ip;
use std::fs;
use base64::{Engine};
use std::path::PathBuf;
use std::env;
use surge_ping::{Client, Config, IcmpPacket, PingIdentifier, PingSequence};
use base64::engine::general_purpose;
use tokio::net::UdpSocket;
use serde::{Deserialize};
use uuid::Uuid;
use std::time::Instant;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::Manager;


#[derive(Debug, Serialize, Deserialize)]
struct DiscoveryMsg {
    app: String,
    device_name: String,
    port: u16,
    instance_id: String,
}


#[derive(Debug, Clone, Serialize)]
struct Pc {
    ip: IpAddr,
    name: String,
}

struct AppState {
    devices: Mutex<HashMap<IpAddr, (Pc, Instant)>>,
    instance_id: String,
}

const PORT: u16 = 5005;

// --- SEND FILE ---
#[tauri::command]
async fn send_file(ip: String, file_path: String, file_name: String) -> Result<String, String> {
    let filename_bytes = file_name.as_bytes();
    let filename_len = filename_bytes.len() as u64;

    let full_address = format!("{}:{}", ip, PORT);
    println!("Connecting to {}...", full_address);

    let mut stream = TcpStream::connect(&full_address)
        .await
        .map_err(|e| format!("Failed to connect to {}: {}", ip, e))?;

    stream.write_u64(filename_len)
        .await
        .map_err(|e| format!("Failed to send length: {}", e))?;

    stream.write_all(filename_bytes)
        .await
        .map_err(|e| format!("Failed to send filename: {}", e))?;

    let mut file = File::open(&file_path)
        .await
        .map_err(|e| format!("Could not open file '{}': {}", file_path, e))?;

    println!("Sending content...");
    let amount_sent = tokio::io::copy(&mut file, &mut stream)
        .await
        .map_err(|e| format!("Transfer interrupted: {}", e))?;

    stream.shutdown().await.map_err(|e| e.to_string())?;

    println!("Success! Sent {} bytes.", amount_sent);

    Ok(format!("Successfully sent {} bytes", amount_sent))
}




async fn broadcast_presence(msg: DiscoveryMsg) {
    let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
    socket.set_broadcast(true).unwrap();
    let data = serde_json::to_vec(&msg).unwrap();

    println!("[DEBUG][BROADCAST] Sending presence every 3s");

    loop {
        if let Err(e) = socket.send_to(&data, "255.255.255.255:5005").await {
            println!("[DEBUG][BROADCAST] Failed to send: {}", e);
        }

        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}


async fn listen_for_devices(state: Arc<AppState>) {
    let socket = UdpSocket::bind("0.0.0.0:5005").await.unwrap();
    let mut buf = [0u8; 1024];
    println!("[DEBUG][LISTENER] UDP listener started on 0.0.0.0:5005");

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, addr)) => {
                if let Ok(msg) = serde_json::from_slice::<DiscoveryMsg>(&buf[..len]) {
                    if msg.app == "vdrop" && msg.instance_id != state.instance_id {
                        let mut devices = state.devices.lock().await;
                        devices.insert(
                            addr.ip(),
                            (Pc { ip: addr.ip(), name: msg.device_name.clone() }, std::time::Instant::now())
                        );
                        println!("[DEBUG][LISTENER] Found device: {} ({})", msg.device_name, addr.ip());
                    }
                }
            }
            Err(e) => {
                println!("[DEBUG][LISTENER] UDP recv error: {}", e);
            }
        }
    }
}

// --- SCAN NETWORK COMMAND ---
#[tauri::command]
async fn scan_network(state: State<'_, AppState>) -> Result<Vec<Pc>, String> {
    let devices = state.devices.lock().await;
    Ok(devices.values().map(|(pc, _)| pc.clone()).collect())
}




pub async fn is_alive(ip: &IpAddr) -> bool {
    // create ICMP client
    let client = match Client::new(&Config::default()) {
        Ok(c) => c,
        Err(_) => return false,
    };

    let payload = [0u8; 16];
    let pinger = client.pinger(*ip, PingIdentifier(0));

    match pinger.await.ping(PingSequence(16), &payload).await {
        Ok((IcmpPacket::V4(_), _)) => true,
        Ok((IcmpPacket::V6(_), _)) => true,
        _ => false,
    }
}


// --- START SERVER TO RECEIVE FILES ---
pub async fn start_receiving() {
    let listener = match TcpListener::bind(format!("0.0.0.0:{}", PORT)).await {
        Ok(l) => {
            println!("[SERVER] Listening on port {}", PORT);
            l
        }
        Err(e) => {
            eprintln!("[SERVER] Failed to bind port: {}", e);
            return;
        }
    };

    loop {
        if let Ok((mut socket, addr)) = listener.accept().await {
            tokio::spawn(async move {
                println!("[INCOMING] Connection from {}", addr);

                let name_len = match socket.read_u64().await {
                    Ok(n) => n,
                    Err(_) => return,
                };

                let _ = tokio::fs::create_dir_all("received").await;

                let mut name_buffer = vec![0u8; name_len as usize];
                if socket.read_exact(&mut name_buffer).await.is_err() {
                    return;
                }

                let filename = String::from_utf8_lossy(&name_buffer);
                let safe_filename = std::path::Path::new(filename.as_ref())
                    .file_name()
                    .map(|s| s.to_str().unwrap())
                    .unwrap_or("unknown_file");

                println!("[INCOMING] Receiving file: '{}'", safe_filename);

                let save_path = format!("received/{}", safe_filename);
                let mut file = match File::create(&save_path).await {
                    Ok(f) => f,
                    Err(e) => {
                        println!("[ERROR] Could not create file: {}", e);
                        return;
                    }
                };

                if let Err(e) = tokio::io::copy(&mut socket, &mut file).await {
                    println!("[ERROR] Transfer failed: {}", e);
                } else {
                    println!("[SUCCESS] Saved '{}'", safe_filename);
                }
            });
        }
    }
}

#[derive(serde::Serialize)]
struct ReceivedFile {
    name: String,
    preview: String,
}


fn received_dir() -> PathBuf {
    let dir = if cfg!(debug_assertions) {
        // Dev mode: use current directory + "received"
        let mut path = std::env::current_dir().unwrap();
        path.push("received");
        path
    } else {
        // Release mode: folder next to executable
        let mut path = std::env::current_exe().unwrap();
        path.pop(); // remove exe
        path.push("received");
        path
    };

    std::fs::create_dir_all(&dir).unwrap();
    println!("Using received_dir: {:?}", dir);
    dir
}

#[tauri::command]
fn get_received() -> Result<Vec<ReceivedFile>, String> {
    let received_dir = received_dir();

    if !received_dir.exists() {
        println!("Directory does not exist!");
        return Ok(vec![]);
    }

    let mut files_vec = Vec::new();

    for entry_result in fs::read_dir(&received_dir).map_err(|e| e.to_string())? {
        let entry = entry_result.map_err(|e| e.to_string())?;
        let path = entry.path();
        println!("Found path: {:?}", path);

        if path.is_file() {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            println!("Found file: {}", name);

            let mut preview = String::new();
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if ext == "png" || ext == "jpg" || ext == "jpeg" {
                    if let Ok(data) = fs::read(&path) {
                        println!("Read {} bytes from file", data.len());
                        preview  = general_purpose::STANDARD.encode(&data,);

                    }
                }
            }

            files_vec.push(ReceivedFile { name, preview });
        }
    }

    println!("Returning {} files", files_vec.len());
    Ok(files_vec)
}


async fn cleanup_devices(state: Arc<AppState>) {
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;

        let mut devices = state.devices.lock().await;
        let before = devices.len();
        devices.retain(|_, (_, last_seen)| last_seen.elapsed() < Duration::from_secs(15));
        let after = devices.len();

        println!("[DEBUG][CLEANUP] Devices before: {}, after: {}", before, after);
    }
}
// --- RUN APP ---
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let instance_id = Uuid::new_v4().to_string();
    let device_name = get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let state = Arc::new(AppState {
        devices: Mutex::new(HashMap::new()),
        instance_id: instance_id.clone(),
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state.clone())
        .setup(move |_app| {
            // clone Arc, now safe to send into 'static tasks
            let state_clone = state.clone();


            println!("[DEBUG] Starting VDrop app");
            println!("[DEBUG] Device name: {}", device_name);
            println!("[DEBUG] Instance ID: {}", instance_id);

            let msg = DiscoveryMsg {
                app: "vdrop".into(),
                device_name,
                port: PORT,
                instance_id,
            };


            tauri::async_runtime::spawn(broadcast_presence(msg));
            tauri::async_runtime::spawn(listen_for_devices(state_clone.clone()));
            tauri::async_runtime::spawn(cleanup_devices(state_clone.clone()));
            tauri::async_runtime::spawn(start_receiving());

            Ok(())
        })

        .invoke_handler(tauri::generate_handler![
            scan_network,
            send_file,
            get_received
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
