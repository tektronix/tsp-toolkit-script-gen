use actix_files as fs;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_ws::{Message, Session};
use futures::StreamExt;
use script_gen_manager::script_component::script::ScriptModel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

use tokio::io::{self, AsyncBufReadExt};
use tokio::signal;
use tokio::sync::watch;

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

async fn ws_index(
    req: HttpRequest,
    body: web::Payload,
    app_state: web::Data<Arc<AppState>>,
) -> Result<HttpResponse, Error> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;
    //msg_stream = msg_stream.max_frame_size(50 * 1024 * 1024); // 50MB

    // Use the app_state here
    {
        let mut session_guard = app_state.session.lock().await;
        *session_guard = Some(session.clone());
    }

    let gen_script_tx = app_state.gen_script_tx.clone();

    let mut chunk_buffers: HashMap<String, Vec<Option<String>>> = HashMap::new();

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        return;
                    }
                }
                Message::Text(mut msg) => {
                    let mut is_chunked = false;
                    {
                        use serde_json::Value;
                        if let Ok(value) = serde_json::from_str::<Value>(&msg) {
                            if let (
                                Some(msg_id),
                                Some(chunk_index),
                                Some(total_chunks),
                                Some(data),
                            ) = (
                                value.get("msg_id").and_then(|v| v.as_str()),
                                value.get("chunk_index").and_then(|v| v.as_u64()),
                                value.get("total_chunks").and_then(|v| v.as_u64()),
                                value.get("data").and_then(|v| v.as_str()),
                            ) {
                                is_chunked = true;
                                let entry = chunk_buffers
                                    .entry(msg_id.to_string())
                                    .or_insert_with(|| vec![None; total_chunks as usize]);
                                entry[chunk_index as usize] = Some(data.to_string());
                                if entry.iter().all(|c| c.is_some()) {
                                    let full_msg = entry
                                        .iter()
                                        .map(|c| c.as_ref().unwrap().as_str())
                                        .collect::<String>();
                                    chunk_buffers.remove(msg_id);
                                    println!(
                                        "Received complete chunked message of size: {} bytes",
                                        full_msg.len()
                                    );
                                    msg = full_msg.into();
                                } else {
                                    println!(
                                        "Received chunk {}/{} for msg_id {}",
                                        chunk_index + 1,
                                        total_chunks,
                                        msg_id
                                    );
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
                    match serde_json::from_str::<IpcData>(&msg) {
                        Ok(ipc_data) => {
                            if ipc_data.request_type == "get_data" {
                                println!("instrument data requested");
                            } else if ipc_data.request_type == "evaluate_data" {
                                let mut data_model = app_state.data_model.lock().await;
                                let response =
                                    data_model.process_data_from_client(ipc_data.json_value);
                                // Send generate script signal
                                if let Err(e) = gen_script_tx.send(()) {
                                    eprintln!("Failed to send signal: {e}");
                                }
                                session.text(response).await.unwrap();
                            } else if ipc_data.request_type == "reallocation" {
                                let mut data_model = app_state.data_model.lock().await;
                                let response = data_model.add_remove_channel(ipc_data);
                                println!("{response}");
                                // Send generate script signal
                                if let Err(e) = gen_script_tx.send(()) {
                                    eprintln!("Failed to send signal: {e}");
                                }
                                session.text(response).await.unwrap();
                            } else if ipc_data.request_type == "open_script" {
                                // Generate script if needed
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

                                session.text(response).await.unwrap();
                            } else {
                                println!("Unknown request type: {}", ipc_data.request_type);
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to deserialize IpcData: {e}");
                        }
                    }
                }
                Message::Close(reason) => {
                    println!("Connection closed: {reason:?}");
                    return;
                }
                _ => (),
            }
        }
        println!("WebSocket message loop ended - connection lost or closed");
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
            } else if trimmed_line.contains("lineFrequency") {
                println!("line frequency received {}", trimmed_line);
                match serde_json::from_str::<serde_json::Value>(trimmed_line) {
                    Ok(json_obj) => {
                        if let Some(freq) = json_obj.get("lineFrequency").and_then(|v| v.as_f64()) {
                            let mut data_model = app_state.data_model.lock().await;
                            data_model
                                .sweep_model
                                .sweep_config
                                .global_parameters
                                .set_line_frequency(freq);
                            println!("Set line frequency to {}", freq);
                        } else {
                            println!(
                                "'lineFrequency' key not found or not a number in JSON: {}",
                                trimmed_line
                            );
                        }
                    }
                    Err(e) => {
                        println!(
                            "Failed to parse line frequency JSON: {} | Error: {}",
                            trimmed_line, e
                        );
                    }
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
