use actix_files as fs;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_ws::{Message, Session};
use futures::StreamExt;
use script_gen_manager::script_component::script::ScriptModel;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

use tokio::io::{self, AsyncBufReadExt};
use tokio::signal;
use tokio::sync::watch;
use tokio::time::{timeout, Duration};

use std::fs::{self as other_fs};
use std::path::Path;

use super::data_model::DataModel;
use crate::back_end::ipc_data::IpcData;

#[derive(Serialize, Deserialize)]
pub struct ScriptPath {
    pub session: String,
    pub folder: String,
}

impl Default for ScriptPath {
    fn default() -> Self {
        Self::new()
    }
}
impl ScriptPath {
    pub fn new() -> Self {
        ScriptPath {
            folder: "C:\\workfolder\\".to_string(),
            session: "sample.tsp".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub session: Arc<Mutex<Option<Session>>>,
    pub data_model: Arc<Mutex<DataModel>>,
    pub gen_script_tx: broadcast::Sender<()>,
    pub work_folder: Arc<Mutex<Option<String>>>,
    pub is_processing: Arc<AtomicBool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let (gen_script_tx, _) = broadcast::channel(100); // Create a broadcast channel
        AppState {
            session: Arc::new(Mutex::new(None)),
            data_model: Arc::new(Mutex::new(DataModel::new())),
            gen_script_tx,
            work_folder: Arc::new(Mutex::new(Option::None)),
            is_processing: Arc::new(AtomicBool::new(false)),
        }
    }

    async fn update_script_name_from_json(&self, json_value: &str) {
        // let new_wf: Script_Path = Script_Path::default();
        // let pat = serde_json::to_string(&new_wf).unwrap();
        // println!("{pat}");
        println!("Updating work folder from JSON: {json_value}");
        let mut work_folder_guard = self.work_folder.lock().await;
        if let Ok(value) = serde_json::from_str::<ScriptPath>(json_value) {
            let filename: String = format!("{}.tsp", value.session.clone());
            let path_file = Path::new(&value.folder).join(filename);
            let folder = path_file.parent().unwrap();
            //println!("Updating work folder to: {:?}", folder.to_string_lossy().to_string());
            // Check if the folder exists and is writable

            if folder.exists() {
                let permissions = folder.metadata().map(|m| m.permissions()).unwrap();
                if permissions.readonly() {
                    println!("Cannot update work folder: folder is read-only");
                    return;
                }
                *work_folder_guard = Some(path_file.to_string_lossy().to_string());
                println!("Work folder updated to: {work_folder_guard:?}");
            } else {
                println!(
                    "Work folder does not exist: {:?}",
                    path_file.to_string_lossy().to_string()
                );
            }
        } else {
            println!("Failed to parse work folder from JSON: {json_value}");
        }
    }
}

// Helper function to send processing status to client
async fn send_processing_status(session: &mut Session, status: &str, details: Option<&str>) {
    let status_msg = json!({
        "type": "processing_status",
        "status": status,
        "details": details,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    });

    if let Ok(msg) = serde_json::to_string(&status_msg) {
        let _ = session.text(msg).await;
    }
}

async fn ws_index(
    req: HttpRequest,
    body: web::Payload,
    app_state: web::Data<Arc<AppState>>,
) -> Result<HttpResponse, Error> {
    println!(
        "New WebSocket connection attempt from: {:?}",
        req.connection_info().peer_addr()
    );
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;
    println!("WebSocket connection established successfully");
    msg_stream = msg_stream.max_frame_size(50 * 1024 * 1024);

    // Use the app_state here
    {
        let mut session_guard = app_state.session.lock().await;
        *session_guard = Some(session.clone());
    }

    let gen_script_tx = app_state.gen_script_tx.clone();
    use std::collections::HashMap;
    let mut chunk_buffers: HashMap<String, Vec<Option<String>>> = HashMap::new();

    actix_web::rt::spawn(async move {
        println!("WebSocket message loop started");
        loop {
            tokio::select! {
                msg_opt = msg_stream.next() => {
                    match msg_opt {
                        Some(Ok(msg)) => {
                            println!("Received WebSocket message: {:?}", msg);
                            match msg {
                                Message::Ping(bytes) => {
                                    if session.pong(&bytes).await.is_err() {
                                        eprintln!("Failed to respond to ping, closing session");
                                        break;
                                    }
                                }
                                Message::Text(mut msg) => {
                                    let mut is_chunked = false;
                                    {
                                        use serde_json::Value;
                                        if let Ok(value) = serde_json::from_str::<Value>(&msg) {
                                            if let (Some(msg_id), Some(chunk_index), Some(total_chunks), Some(data)) = (
                                                value.get("msg_id").and_then(|v| v.as_str()),
                                                value.get("chunk_index").and_then(|v| v.as_u64()),
                                                value.get("total_chunks").and_then(|v| v.as_u64()),
                                                value.get("data").and_then(|v| v.as_str()),
                                            ) {
                                                is_chunked = true;
                                                let entry = chunk_buffers.entry(msg_id.to_string())
                                                    .or_insert_with(|| vec![None; total_chunks as usize]);

                                                entry[chunk_index as usize] = Some(data.to_string());

                                                if entry.iter().all(|c| c.is_some()) {
                                                    let full_msg = entry.iter().map(|c| c.as_ref().unwrap().as_str()).collect::<String>();
                                                    chunk_buffers.remove(msg_id);
                                                    println!("Received complete chunked message of size: {} bytes", full_msg.len());
                                                    msg = full_msg.into();
                                                } else {
                                                    println!("Received chunk {}/{} for msg_id {}", chunk_index+1, total_chunks, msg_id);
                                                    continue;
                                                }
                                            }
                                        }
                                    }
                                    // --- End chunked message reassembly logic ---
                                    // Only fall through to normal processing if not a chunked message or if chunk is complete
                                    if is_chunked && msg.is_empty() {
                                        continue;
                                    }
                                    if app_state.is_processing.load(Ordering::Acquire) {
                                        println!("Server is busy processing another request, rejecting new request");
                                        let busy_response = json!({
                                            "error": "Server busy",
                                            "message": "Server is currently processing another request. Please wait and try again.",
                                            "status": "busy"
                                        });
                                        if let Ok(response_str) = serde_json::to_string(&busy_response) {
                                            let _ = session.text(response_str).await;
                                        }
                                        continue;
                                    }

                                    app_state.is_processing.store(true, Ordering::Release);

                                    const MAX_MESSAGE_SIZE: usize = 50 * 1024 * 1024; // 50MB limit
                                    if msg.len() > MAX_MESSAGE_SIZE {
                                        eprintln!("Message too large: {} bytes (max: {} bytes)", msg.len(), MAX_MESSAGE_SIZE);
                                        let error_response = json!({
                                            "error": "Message size exceeds maximum limit",
                                            "max_size_mb": MAX_MESSAGE_SIZE / (1024 * 1024),
                                            "received_size_mb": msg.len() / (1024 * 1024)
                                        });
                                        if let Ok(response_str) = serde_json::to_string(&error_response) {
                                            let _ = session.text(response_str).await;
                                        }
                                        app_state.is_processing.store(false, Ordering::Release);
                                        continue;
                                    }

                                    let msg_len = msg.len();
                                    println!("Processing message of size: {} bytes", msg_len);

                                    if msg_len > 1024 * 1024 {
                                        send_processing_status(&mut session, "parsing", Some(&format!("Parsing large JSON message ({} MB)", msg_len / (1024 * 1024)))).await;
                                    }

                                    let parse_result = if msg_len > 1024 * 1024 {
                                        match timeout(
                                            Duration::from_secs(30),
                                            tokio::task::spawn_blocking(move || serde_json::from_str::<IpcData>(&msg))
                                        ).await {
                                            Ok(Ok(result)) => result,
                                            Ok(Err(parse_error)) => {
                                                eprintln!("JSON parsing failed: {parse_error}");
                                                app_state.is_processing.store(false, Ordering::Release);
                                                let error_response = json!({
                                                    "error": "JSON parsing failed",
                                                    "details": parse_error.to_string()
                                                });
                                                if let Ok(response_str) = serde_json::to_string(&error_response) {
                                                    let _ = session.text(response_str).await;
                                                }
                                                continue;
                                            }
                                            Err(_timeout_error) => {
                                                eprintln!("JSON parsing timed out after 30 seconds");
                                                app_state.is_processing.store(false, Ordering::Release);
                                                let error_response = json!({
                                                    "error": "JSON parsing timeout",
                                                    "details": "JSON parsing took too long and was cancelled"
                                                });
                                                if let Ok(response_str) = serde_json::to_string(&error_response) {
                                                    let _ = session.text(response_str).await;
                                                }
                                                continue;
                                            }
                                        }
                                    } else {
                                        serde_json::from_str::<IpcData>(&msg)
                                    };

                                    match parse_result {
                                        Ok(ipc_data) => {
                                            let request_type = ipc_data.request_type.clone();
                                            println!("Processing request type: {}", request_type);
                                            send_processing_status(&mut session, "processing", Some(&format!("Processing {} request", request_type))).await;
                                            let start_time = std::time::Instant::now();
                                            match request_type.as_str() {
                                                "get_data" => {
                                                    println!("instrument data requested");
                                                    app_state.is_processing.store(false, Ordering::Release);
                                                }
                                                "evaluate_data" => {
                                                    let app_state_clone = app_state.clone();
                                                    let gen_script_tx_clone = gen_script_tx.clone();
                                                    match timeout(
                                                        Duration::from_secs(120),
                                                        tokio::task::spawn_blocking(move || {
                                                            let rt = tokio::runtime::Handle::current();
                                                            rt.block_on(async {
                                                                let mut data_model = app_state_clone.data_model.lock().await;
                                                                
                                                                //println!("processed data from client {response}");
                                                                data_model.process_data_from_client(ipc_data.json_value)
                                                            })
                                                        })
                                                    ).await {
                                                        Ok(Ok(response)) => {
                                                            let processing_time = start_time.elapsed();
                                                            println!("Processing completed in: {:?}", processing_time);
                                                            tokio::spawn(async move {
                                                                if let Err(e) = gen_script_tx_clone.send(()) {
                                                                    eprintln!("Failed to send signal: {e}");
                                                                }
                                                            });
                                                            send_processing_status(&mut session, "complete", Some(&format!("Processing completed in {:.2}s", processing_time.as_secs_f64()))).await;
                                                            if let Err(e) = session.text(response).await {
                                                                eprintln!("Failed to send response: {e}");
                                                            }
                                                            app_state.is_processing.store(false, Ordering::Release);
                                                        }
                                                        Ok(Err(e)) => {
                                                            eprintln!("Processing task failed: {e}");
                                                            let error_response = json!({
                                                                "error": "Processing failed",
                                                                "details": e.to_string()
                                                            });
                                                            if let Ok(response_str) = serde_json::to_string(&error_response) {
                                                                let _ = session.text(response_str).await;
                                                            }
                                                            app_state.is_processing.store(false, Ordering::Release);
                                                        }
                                                        Err(_timeout_error) => {
                                                            eprintln!("Data processing timed out after 2 minutes");
                                                            let error_response = json!({
                                                                "error": "Processing timeout",
                                                                "details": "Data processing took too long and was cancelled"
                                                            });
                                                            if let Ok(response_str) = serde_json::to_string(&error_response) {
                                                                let _ = session.text(response_str).await;
                                                            }
                                                            app_state.is_processing.store(false, Ordering::Release);
                                                        }
                                                    }
                                                }
                                                "reallocation" => {
                                                    let app_state_clone = app_state.clone();
                                                    let gen_script_tx_clone = gen_script_tx.clone();
                                                    match timeout(
                                                        Duration::from_secs(60),
                                                        tokio::task::spawn_blocking(move || {
                                                            let rt = tokio::runtime::Handle::current();
                                                            rt.block_on(async {
                                                                let mut data_model = app_state_clone.data_model.lock().await;
                                                                let response = data_model.add_remove_channel(ipc_data);
                                                                println!("{response}");
                                                                response
                                                            })
                                                        })
                                                    ).await {
                                                        Ok(Ok(response)) => {
                                                            let processing_time = start_time.elapsed();
                                                            println!("Reallocation completed in: {:?}", processing_time);
                                                            tokio::spawn(async move {
                                                                if let Err(e) = gen_script_tx_clone.send(()) {
                                                                    eprintln!("Failed to send signal: {e}");
                                                                }
                                                            });
                                                            send_processing_status(&mut session, "complete", Some(&format!("Reallocation completed in {:.2}s", processing_time.as_secs_f64()))).await;
                                                            if let Err(e) = session.text(response).await {
                                                                eprintln!("Failed to send response: {e}");
                                                            }
                                                            app_state.is_processing.store(false, Ordering::Release);
                                                        }
                                                        Ok(Err(e)) => {
                                                            eprintln!("Reallocation task failed: {e}");
                                                            let error_response = json!({
                                                                "error": "Reallocation failed",
                                                                "details": e.to_string()
                                                            });
                                                            if let Ok(response_str) = serde_json::to_string(&error_response) {
                                                                let _ = session.text(response_str).await;
                                                            }
                                                            app_state.is_processing.store(false, Ordering::Release);
                                                        }
                                                        Err(_timeout_error) => {
                                                            eprintln!("Reallocation timed out after 1 minute");
                                                            let error_response = json!({
                                                                "error": "Reallocation timeout",
                                                                "details": "Reallocation took too long and was cancelled"
                                                            });
                                                            if let Ok(response_str) = serde_json::to_string(&error_response) {
                                                                let _ = session.text(response_str).await;
                                                            }
                                                            app_state.is_processing.store(false, Ordering::Release);
                                                        }
                                                    }
                                                }
                                                "open_script" => {
                                                    if let Err(e) = gen_script_tx.send(()) {
                                                        eprintln!("Failed to send signal: {e}");
                                                    }
                                                    let res = IpcData {
                                                        request_type: "open_script".to_string(),
                                                        additional_info: "".to_string(),
                                                        json_value: "{}".to_string(),
                                                    };
                                                    let response = serde_json::to_string(&res)
                                                        .expect("Failed to serialize response");
                                                    let processing_time = start_time.elapsed();
                                                    send_processing_status(&mut session, "complete", Some(&format!("Script generation completed in {:.2}s", processing_time.as_secs_f64()))).await;
                                                    if let Err(e) = session.text(response).await {
                                                        eprintln!("Failed to send open_script response: {e}");
                                                    }
                                                    app_state.is_processing.store(false, Ordering::Release);
                                                }
                                                _ => {
                                                    println!("Unknown request type: {}", request_type);
                                                    let error_response = json!({
                                                        "error": "Unknown request type",
                                                        "request_type": request_type
                                                    });
                                                    if let Ok(response_str) = serde_json::to_string(&error_response) {
                                                        let _ = session.text(response_str).await;
                                                    }
                                                    app_state.is_processing.store(false, Ordering::Release);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to deserialize IpcData: {e}");
                                            eprintln!("Message size was: {} bytes", msg_len);
                                            let error_response = json!({
                                                "error": "Failed to parse JSON message",
                                                "details": e.to_string(),
                                                "message_size_bytes": msg_len
                                            });
                                            if let Ok(response_str) = serde_json::to_string(&error_response) {
                                                let _ = session.text(response_str).await;
                                            }
                                            app_state.is_processing.store(false, Ordering::Release);
                                        }
                                    }
                                }
                                Message::Close(reason) => {
                                    println!("Connection closed by client: {reason:?}");
                                    app_state.is_processing.store(false, Ordering::Release);
                                    break;
                                }
                                Message::Pong(_) => {
                                    // Optionally log pong
                                }
                                _ => (),
                            }
                        }
                        Some(Err(e)) => {
                            eprintln!("WebSocket error: {e}");
                            break;
                        }
                        None => {
                            eprintln!("WebSocket stream ended (client disconnected)");
                            break;
                        }
                    }
                }
            }
        }
        println!("WebSocket message loop ended - connection lost or closed");
        app_state.is_processing.store(false, Ordering::Release);
    });

    Ok(response)
}

async fn serve_index_html() -> Result<HttpResponse, Error> {
    // Get the path of the executable
    let exe_path =
        std::env::current_exe().expect("should be able to get path of server executable");

    // Get the directory of the executable (this will be `script-gen-win32-x64/bin`)
    let exe_dir = exe_path
        .parent()
        .expect("should be able to get directory of server executable");

    //browser directory and kic-script-gen.exe are on the same level in npm package
    let browser_dir = exe_dir.join("browser");

    // Path to the HTML file
    let html_path = browser_dir.join("index.html");

    // Read the HTML file
    let html_content = other_fs::read_to_string(&html_path).map_err(|e| {
        eprintln!("Failed to read HTML file: {e}");
        actix_web::error::ErrorInternalServerError("Failed to load HTML")
    })?;

    // Rewrite resource URLs
    let base_url = "http://127.0.0.1:27950";
    let modified_html = html_content
        .replace("src=\"", &format!("src=\"{base_url}/"))
        .replace("href=\"", &format!("href=\"{base_url}/"));

    // Return the modified HTML
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(modified_html))
}

pub async fn start_web_server(
    app_state: Arc<AppState>,
    mut shutdown_rx: watch::Receiver<()>,
) -> std::io::Result<()> {
    let server = HttpServer::new(move || {
        let exe_path =
            std::env::current_exe().expect("should be able to get path of server executable");
        let exe_dir = exe_path
            .parent()
            .expect("should be able to get directory of server executable");
        let browser_dir = exe_dir.join("browser");
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .route("/", web::get().to(serve_index_html))
            .route("/ws", web::get().to(ws_index))
            .service(fs::Files::new("/", browser_dir).index_file("index.html"))
            .wrap(
                actix_cors::Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec!["Content-Type"]),
            )
    })
    .bind(("127.0.0.1", 27950))?
    .run();

    println!("WebSocket server started on http://127.0.0.1:27950");
    println!("WebSocket endpoint available at ws://127.0.0.1:27950/ws");

    tokio::select! {
        res = server => res,
        _ = shutdown_rx.changed() => {
            println!("Shutdown signal received, stopping server...");
            Ok(())
        },
    }
}

pub async fn start(mut script_model: ScriptModel) -> anyhow::Result<()> {
    let app_state = Arc::new(AppState::new());

    let (shutdown_tx, shutdown_rx) = watch::channel(());
    let server = start_web_server(app_state.clone(), shutdown_rx.clone());

    // Clone the event receiver for generating script
    let mut gen_script_rx = app_state.gen_script_tx.subscribe();

    // Spawn a task to listen to generate script event
    {
        let app_state_clone = app_state.clone();
        tokio::spawn(async move {
            while let Ok(()) = gen_script_rx.recv().await {
                println!("Signal received to start script generation!");
                let data_model = app_state_clone.data_model.lock().await;
                let work_folder_guard = app_state_clone.work_folder.lock().await;
                let work_folder = work_folder_guard
                    .as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or("C:\\default.tsp");
                script_model.to_script(&data_model.sweep_model.sweep_config, work_folder);
            }
        });
    }

    // Spawn a task to listen for shutdown signal (e.g., Ctrl+C)
    tokio::spawn({
        let shutdown_tx = shutdown_tx.clone();
        async move {
            signal::ctrl_c().await.expect("Failed to listen for event");
            println!("Received Ctrl+C, shutting down...");
            let _ = shutdown_tx.send(());
        }
    });

    // Spawn a task to listen to stdin
    tokio::spawn(async move {
        let stdin = io::stdin();
        let mut reader = io::BufReader::new(stdin).lines();

        let app_state = app_state.clone();

        while let Some(line) = reader.next_line().await.unwrap() {
            //println!("Received from stdin: {line}");
            let trimmed_line = line.trim();
            if trimmed_line == "shutdown" {
                println!("Received shutdown command from stdin, shutting down...");
                let _ = shutdown_tx.send(());
                break;
            } else if trimmed_line.contains("session") && trimmed_line.contains("folder") {
                app_state.update_script_name_from_json(trimmed_line).await;
            } else if trimmed_line.contains("systems") {
                let mut data_model = app_state.data_model.lock().await;
                let response = data_model.process_system_config(trimmed_line);
                println!("{response}");
                // Send generate script signal
                if !response.contains("error") {
                    if let Err(e) = app_state.gen_script_tx.send(()) {
                        eprintln!("Failed to send signal: {e}");
                    }
                }
                let mut session = app_state.session.lock().await;
                if let Some(session) = session.as_mut() {
                    session.text(response).await.unwrap();
                }
            } else if trimmed_line.contains("refresh") {
                println!("instrument data requested"); // refreshing by initiating session again does not affect the JSON state
            } else if trimmed_line.contains("reset") {
                let mut data_model = app_state.data_model.lock().await;
                let response = data_model.reset_sweep_config();
                let mut session = app_state.session.lock().await;
                if let Some(session) = session.as_mut() {
                    session.text(response).await.unwrap();
                }
                println!("instrument data requested"); // getting the system configuration for new session
            } else if trimmed_line.contains("request_type") {
                // both the functions process_data_from_client and process_data_from_saved_config expect sweep_config, process_data_from_client receives sweep_config directly from client
                // process_data_from_saved_config receives sweep_model from saved JSON, this sweep_model has the sweep_config
                let mut data_model = app_state.data_model.lock().await;
                match serde_json::from_str::<IpcData>(trimmed_line) {
                    Ok(ipc_data) => {
                        match serde_json::from_str::<serde_json::Value>(&ipc_data.json_value) {
                            Ok(json_obj) => {
                                if let Some(sweep_model) = json_obj.get("sweep_model") {
                                    if let Ok(sweep_model_str) = serde_json::to_string(sweep_model)
                                    {
                                        let response = data_model
                                            .process_data_from_saved_config(sweep_model_str);
                                        println!("processed data from saved config {response}");
                                        let mut session = app_state.session.lock().await;
                                        if let Some(session) = session.as_mut() {
                                            session.text(response).await.unwrap();
                                        }
                                    } else {
                                        eprintln!("Failed to serialize sweep_model to string");
                                    }
                                } else {
                                    eprintln!("'sweep_model' field not found in json_value");
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to parse json_value as JSON: {e}");
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to deserialize IpcData from stdin: {e}");
                    }
                }
            }
        }
    });

    //run()?;
    server.await?;
    Ok(())
}
