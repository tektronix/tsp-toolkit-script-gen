use actix_files as fs;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_ws::{Message, Session};
use futures::StreamExt;
use script_gen_manager::script_component::script::ScriptModel;
use serde::{Deserialize, Serialize};
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
        println!("Updating work folder from JSON: {}", json_value);
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
                println!("Work folder updated to: {:?}", work_folder_guard);
            } else {
                println!(
                    "Work folder does not exist: {:?}",
                    path_file.to_string_lossy().to_string()
                );
            }
        } else {
            println!("Failed to parse work folder from JSON: {}", json_value);
        }
    }
}

async fn ws_index(
    req: HttpRequest,
    body: web::Payload,
    app_state: web::Data<Arc<AppState>>,
) -> Result<HttpResponse, Error> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    // Use the app_state here
    {
        let mut session_guard = app_state.session.lock().await;
        *session_guard = Some(session.clone());
    }

    let gen_script_tx = app_state.gen_script_tx.clone();

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        return;
                    }
                }
                Message::Text(msg) => {
                    println!("Received input from client in session: {msg}");
                    match serde_json::from_str::<IpcData>(&msg) {
                        Ok(ipc_data) => {
                            if ipc_data.request_type == "get_data" {
                                println!("instrument data requested");
                            } else if ipc_data.request_type == "evaluate_data" {
                                let mut data_model = app_state.data_model.lock().await;
                                let response =
                                    data_model.process_data_from_client(ipc_data.json_value);
                                println!("processed data from client");
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
                println!("{}", response);
                // Send generate script signal
                if !response.contains("error") {
                    if let Err(e) = app_state.gen_script_tx.send(()) {
                        eprintln!("Failed to send signal: {}", e);
                    }
                }
                let mut session = app_state.session.lock().await;
                if let Some(session) = session.as_mut() {
                    session.text(response).await.unwrap();
                }
            } else if trimmed_line.contains("reset") {
                let mut data_model = app_state.data_model.lock().await;
                let response = data_model.reset_sweep_config();
                let mut session = app_state.session.lock().await;
                if let Some(session) = session.as_mut() {
                    session.text(response).await.unwrap();
                }
                println!("instrument data requested"); // getting the system configuration for new session
            } else if trimmed_line.contains("request_type") {
                // if receiving the saved JSON
                let mut data_model = app_state.data_model.lock().await;
                match serde_json::from_str::<IpcData>(trimmed_line) {
                    Ok(ipc_data) => {
                        let response =
                            data_model.process_data_from_saved_config(ipc_data.json_value);
                        println!("processed data from saved config");
                        // Send data to the client and generate script signal
                        let mut session = app_state.session.lock().await;
                        if let Some(session) = session.as_mut() {
                            session.text(response).await.unwrap();
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to deserialize IpcData from stdin: {}", e);
                    }
                }
            }
        }
    });

    //run()?;
    server.await?;
    Ok(())
}
