use tauri::State;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::fs::File;
use tokio::time::{timeout, Duration};
use tokio::task;
use tokio::sync::Mutex;
use tokio::process::Command;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
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




#[derive(Debug, Clone, Serialize)]
struct Pc {
    ip: IpAddr,
    name: String,
}

struct AppState {
    devices: Mutex<Vec<Pc>>,
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

// --- SCAN NETWORK COMMAND ---
#[tauri::command]
async fn scan_network(state: State<'_, AppState>) -> Result<Vec<Pc>, String> {
    println!("[SCAN] Starting network scan...");
    let scanned = internal_scan_logic().await.map_err(|e| e.to_string())?;
    println!("[SCAN] Initial scan found {} devices", scanned.len());
    
    let timeout_duration = Duration::from_millis(200);

    let mut tasks = futures::stream::FuturesUnordered::new();

    for pc in scanned {
        let pc_clone = pc.clone();
        tasks.push(task::spawn(async move {
            let addr = SocketAddr::new(pc_clone.ip, PORT);
            if timeout(timeout_duration, TcpStream::connect(addr))
                .await
                .ok()
                .and_then(|r| r.ok())
                .is_some()
            {
                println!("[SCAN] {} has our app running!", pc_clone.ip);
                Some(pc_clone)
            } else {
                println!("[SCAN] {} doesn't have our app", pc_clone.ip);
                None
            }
        }));
    }

    let mut final_list = Vec::new();
    while let Some(result) = tasks.next().await {
        if let Ok(Some(pc)) = result {
            final_list.push(pc);
        }
    }

    println!("[SCAN] Final list: {} devices with our app", final_list.len());
    for pc in &final_list {
        println!("[SCAN] Device: {} ({})", pc.name, pc.ip);
    }

    let mut guard = state.devices.lock().await;
    *guard = final_list.clone();

    Ok(final_list)
}

// --- INTERNAL NETWORK SCAN ---
async fn internal_scan_logic() -> Result<Vec<Pc>, Box<dyn std::error::Error>> {
    let local_ip_addr = local_ip()?;
    println!("[SCAN] Local IP detected: {}", local_ip_addr);
    
    let (subnet_a, subnet_b, subnet_c) = match local_ip_addr {
        IpAddr::V4(ipv4) => {
            let octets = ipv4.octets();
            (octets[0], octets[1], octets[2])
        }
        _ => (192, 168, 1),
    };

    println!("[SCAN] Scanning subnet: {}.{}.{}.0/24", subnet_a, subnet_b, subnet_c);
    
    let self_name = get()?.to_string_lossy().to_string();
    println!("[SCAN] Local hostname: {}", self_name);
    
    let mut tasks = futures::stream::FuturesUnordered::new();

    for i in 1..=254 {
        let ip = IpAddr::V4(Ipv4Addr::new(subnet_a, subnet_b, subnet_c, i));
        // for deleting ourselves from users lists
        //let self_name_clone = self_name.clone();
        tasks.push(task::spawn(async move {
            let alive = is_alive(&ip).await;
            if alive {
                println!("[SCAN] Found alive host: {}", ip);
                let name = get_hostname(&ip).await;
                println!("[SCAN] Hostname resolved: {} -> {}", ip, name);
                return Ok::<Option<Pc>, Box<dyn std::error::Error + Send + Sync>>(Some(Pc { ip, name }));
            }
            Ok(None)
        }));
    }

    let mut pcs = Vec::new();
    while let Some(result) = tasks.next().await {
        if let Ok(Ok(Some(pc))) = result {
            pcs.push(pc);
        }
    }

    println!("[SCAN] Scan complete. Found {} devices", pcs.len());
    Ok(pcs)
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

async fn get_hostname(ip: &IpAddr) -> String {
    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = timeout(
            Duration::from_secs(1),
            Command::new("nslookup").arg(ip.to_string()).output()
        )
        .await
        {
            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(name) = stdout
                    .lines()
                    .find(|l| l.contains("Name:"))
                    .and_then(|l| l.split_whitespace().last())
                {
                    return name.to_string();
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(output) = timeout(
            Duration::from_secs(1),
            Command::new("host").arg(ip.to_string()).output()
        )
        .await
        {
            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = stdout.lines().next() {
                    if line.contains("domain name pointer") {
                        if let Some(name) = line.split("pointer").nth(1) {
                            return name.trim().trim_end_matches('.').to_string();
                        }
                    }
                }
            }
        }
    }

    format!("Device-{}", ip)
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

// --- RUN APP ---
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            devices: Mutex::new(Vec::new()),
        })
        .setup(|_app| {
            tauri::async_runtime::spawn(async move {
                start_receiving().await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![scan_network, send_file,get_received])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
