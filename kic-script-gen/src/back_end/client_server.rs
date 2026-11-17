use actix_files as fs;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_ws::{Message, Session};
use futures::StreamExt;
use script_gen_manager::script_component::script::ScriptModel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};

use tokio::io::{self, AsyncBufReadExt};
use tokio::signal;
use tokio::sync::watch;

use std::fs::{self as other_fs};
use std::path::Path;

use super::data_model::DataModel;
use crate::back_end::ipc_data::IpcData;
use script_gen_manager::model::sweep_data::status_msg::{StatusMsg, StatusType};

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
    pub gen_script_tx: mpsc::Sender<GenerationJob>,
    gen_script_rx: Arc<Mutex<Option<mpsc::Receiver<GenerationJob>>>>,
    pub work_folder: Arc<Mutex<Option<String>>>,
}

#[derive(Default)]
pub struct GenerationJob {
    completion: Option<oneshot::Sender<Result<(), String>>>,
    // One-off destination that replaces the session's output path for this job only
    target_path: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let (gen_script_tx, gen_script_rx) = mpsc::channel(100);
        AppState {
            session: Arc::new(Mutex::new(None)),
            data_model: Arc::new(Mutex::new(DataModel::new())),
            gen_script_tx,
            gen_script_rx: Arc::new(Mutex::new(Some(gen_script_rx))),
            work_folder: Arc::new(Mutex::new(Option::None)),
        }
    }

    async fn generate_scripts(
        mut script_model: ScriptModel,
        app_state: Arc<AppState>,
        mut requests: mpsc::Receiver<GenerationJob>,
    ) {
        while let Some(job) = requests.recv().await {
            println!("Signal received to start script generation!");
            let result = {
                let data_model = app_state.data_model.lock().await;
                let work_folder_guard = app_state.work_folder.lock().await;
                match job.target_path.as_deref().or(work_folder_guard.as_deref()) {
                    None => Err("No script output path is configured".to_string()),
                    Some(path) => script_model
                        .to_script(&data_model.sweep_model.sweep_config, path)
                        .map_err(|error| format!("{error:#}")),
                }
            };
            if let Err(error) = &result {
                eprintln!("Script generation failed: {error}");
            }
            if let Some(completion) = job.completion {
                if completion.send(result).is_err() {
                    eprintln!("Script generation requester disconnected before completion");
                }
            }
        }
    }

    fn generation_error(request_type: String, error: String) -> IpcData {
        eprintln!("{request_type} failed: {error}");
        IpcData {
            request_type: "script_generation_error".to_string(),
            additional_info: request_type,
            json_value: serde_json::to_string(&StatusMsg::new(StatusType::Error, error, None))
                .expect("Failed to serialize generation error"),
        }
    }

    /// Queues a generation job whose result is reported after the caller's own response,
    /// so data updates are never delayed by the script write.
    async fn queue_background_generation(
        gen_script_tx: &mpsc::Sender<GenerationJob>,
    ) -> Result<oneshot::Receiver<Result<(), String>>, String> {
        let (completion, completed) = oneshot::channel();
        gen_script_tx
            .send(GenerationJob {
                completion: Some(completion),
                target_path: None,
            })
            .await
            .map(|()| completed)
            .map_err(|_| "Script generation worker is unavailable".to_string())
    }

    async fn report_generation_failure(
        queued: Result<oneshot::Receiver<Result<(), String>>, String>,
        request_type: String,
        mut session: Session,
    ) {
        let result = match queued {
            Ok(completed) => completed.await.unwrap_or_else(|_| {
                Err("Script generation worker stopped before completion".to_string())
            }),
            Err(error) => Err(error),
        };
        if let Err(error) = result {
            let response = serde_json::to_string(&Self::generation_error(request_type, error))
                .expect("Failed to serialize response");
            if session.text(response).await.is_err() {
                eprintln!("Failed to deliver script generation error: session closed");
            }
        }
    }

    async fn generation_response(app_state: &AppState, request: IpcData) -> IpcData {
        // Save As writes straight to the path the user picked; Open Script uses the session path
        let target_path = if request.request_type == "save_as_generated_script" {
            if request.additional_info.trim().is_empty() {
                return Self::generation_error(
                    request.request_type,
                    "No Save As destination was provided".to_string(),
                );
            }
            Some(request.additional_info.clone())
        } else {
            None
        };
        let (completion, completed) = oneshot::channel();
        let result = match app_state
            .gen_script_tx
            .send(GenerationJob {
                completion: Some(completion),
                target_path,
            })
            .await
        {
            Ok(()) => completed.await.unwrap_or_else(|_| {
                Err("Script generation worker stopped before completion".to_string())
            }),
            Err(_) => Err("Script generation worker is unavailable".to_string()),
        };
        match result {
            Ok(()) => IpcData {
                request_type: request.request_type,
                additional_info: request.additional_info,
                json_value: "{}".to_string(),
            },
            Err(error) => Self::generation_error(request.request_type, error),
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
                *work_folder_guard = Some(path_file.to_string_lossy().to_string());
                println!("Work folder updated to: {work_folder_guard:?}");
            } else {
                let status_msg = StatusMsg::new(
                    StatusType::Error,
                    format!(
                        "The folder is read-only (OR) Work folder does not exist: {:?}",
                        path_file.to_string_lossy().to_string()
                    ),
                    None,
                );
                println!(
                    "{}",
                    serde_json::to_string(&status_msg)
                        .unwrap_or_else(|_| status_msg.message.clone())
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
                                drop(data_model);
                                let queued =
                                    AppState::queue_background_generation(&gen_script_tx).await;
                                session.text(response).await.unwrap();
                                actix_web::rt::spawn(AppState::report_generation_failure(
                                    queued,
                                    ipc_data.request_type,
                                    session.clone(),
                                ));
                            } else if ipc_data.request_type == "reallocation" {
                                let mut data_model = app_state.data_model.lock().await;
                                let request_type = ipc_data.request_type.clone();
                                let response = data_model.add_remove_channel(ipc_data);
                                drop(data_model);
                                println!("{response}");
                                let queued =
                                    AppState::queue_background_generation(&gen_script_tx).await;
                                session.text(response).await.unwrap();
                                actix_web::rt::spawn(AppState::report_generation_failure(
                                    queued,
                                    request_type,
                                    session.clone(),
                                ));
                            } else if ipc_data.request_type == "save_as_generated_script"
                                || ipc_data.request_type == "open_script"
                            {
                                let res = AppState::generation_response(&app_state, ipc_data).await;
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

pub async fn start(script_model: ScriptModel) -> anyhow::Result<()> {
    let app_state = Arc::new(AppState::new());

    let (shutdown_tx, shutdown_rx) = watch::channel(());
    let server = start_web_server(app_state.clone(), shutdown_rx.clone());

    // The generation queue has a single worker.
    let gen_script_rx = app_state
        .gen_script_rx
        .lock()
        .await
        .take()
        .expect("Generation worker receiver already taken");

    // Spawn a task to listen to generate script event
    {
        let app_state_clone = app_state.clone();
        tokio::spawn(AppState::generate_scripts(
            script_model,
            app_state_clone,
            gen_script_rx,
        ));
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
            if trimmed_line.contains("shutdown") {
                println!("Received shutdown command from stdin, shutting down...");
                let _ = shutdown_tx.send(());
                break;
            } else if trimmed_line.contains("session") && trimmed_line.contains("folder") {
                app_state.update_script_name_from_json(trimmed_line).await;
            } else if trimmed_line.contains("systems") {
                let mut data_model = app_state.data_model.lock().await;
                let response = data_model.process_system_config(trimmed_line);
                drop(data_model);
                println!("{response}");
                // Send generate script signal
                if !response.contains("error") {
                    if let Err(e) = app_state.gen_script_tx.send(GenerationJob::default()).await {
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

#[cfg(test)]
mod tests {
    use super::*;
    use script_gen_manager::catalog::Catalog;
    use std::future::Future;
    use std::task::Poll;

    fn request(action: &str) -> IpcData {
        IpcData {
            request_type: action.to_string(),
            additional_info: if action == "save_as_generated_script" {
                "C:\\out\\a.tsp".to_string()
            } else {
                String::new()
            },
            json_value: "{}".to_string(),
        }
    }

    fn assert_pending(future: std::pin::Pin<&mut impl Future>) {
        let waker = futures::task::noop_waker();
        assert!(matches!(
            future.poll(&mut std::task::Context::from_waker(&waker)),
            Poll::Pending
        ));
    }

    #[actix_web::test]
    async fn websocket_actions_do_not_respond_before_completion() {
        use actix_web::{body::MessageBody, http::StatusCode, test};
        use futures::future::poll_fn;

        for action in ["save_as_generated_script", "open_script"] {
            let state = Arc::new(AppState::new());
            let mut jobs = state.gen_script_rx.lock().await.take().unwrap();
            let app = test::init_service(
                App::new()
                    .app_data(web::Data::new(state))
                    .route("/ws", web::get().to(ws_index)),
            )
            .await;
            let message = serde_json::to_string(&request(action)).unwrap();
            let mask = [1, 2, 3, 4];
            let length = u8::try_from(message.len()).unwrap();
            assert!(length < 126);
            let mut frame = vec![0x81, 0x80 | length];
            frame.extend_from_slice(&mask);
            frame.extend(
                message
                    .bytes()
                    .enumerate()
                    .map(|(index, byte)| byte ^ mask[index % mask.len()]),
            );
            let request = test::TestRequest::get()
                .uri("/ws")
                .insert_header(("connection", "upgrade"))
                .insert_header(("upgrade", "websocket"))
                .insert_header(("sec-websocket-version", "13"))
                .insert_header(("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ=="))
                .set_payload(frame)
                .to_request();
            let response = test::call_service(&app, request).await;
            assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
            let job = tokio::time::timeout(std::time::Duration::from_secs(5), jobs.recv())
                .await
                .unwrap()
                .unwrap();
            let mut body = std::pin::pin!(response.into_body());
            let mut next_frame = std::pin::pin!(poll_fn(|cx| body.as_mut().poll_next(cx)));
            assert_pending(next_frame.as_mut());
            job.completion.unwrap().send(Ok(())).unwrap();
            let bytes = tokio::time::timeout(std::time::Duration::from_secs(5), next_frame)
                .await
                .unwrap()
                .unwrap()
                .unwrap();
            let mut expected = vec![0x81, length];
            expected.extend_from_slice(message.as_bytes());
            assert_eq!(bytes.as_ref(), expected);
        }
    }

    #[tokio::test]
    async fn actions_wait_for_their_own_completion() {
        let state = AppState::new();
        let mut jobs = state.gen_script_rx.lock().await.take().unwrap();
        state
            .gen_script_tx
            .send(GenerationJob::default())
            .await
            .unwrap();
        let mut save = std::pin::pin!(AppState::generation_response(
            &state,
            request("save_as_generated_script")
        ));
        let mut open = std::pin::pin!(AppState::generation_response(
            &state,
            request("open_script")
        ));
        assert_pending(save.as_mut());
        assert_pending(open.as_mut());

        assert!(jobs.recv().await.unwrap().completion.is_none());
        assert_pending(save.as_mut());
        assert_pending(open.as_mut());
        let save_job = jobs.recv().await.unwrap();
        let open_job = jobs.recv().await.unwrap();
        open_job.completion.unwrap().send(Ok(())).unwrap();
        assert_eq!(open.await.request_type, "open_script");
        assert_pending(save.as_mut());
        save_job.completion.unwrap().send(Ok(())).unwrap();
        assert_eq!(save.await.request_type, "save_as_generated_script");
    }

    #[tokio::test]
    async fn failed_completion_returns_an_error_instead_of_an_action() {
        for action in ["save_as_generated_script", "open_script"] {
            let state = AppState::new();
            let mut jobs = state.gen_script_rx.lock().await.take().unwrap();
            let mut response =
                std::pin::pin!(AppState::generation_response(&state, request(action)));
            assert_pending(response.as_mut());
            jobs.recv()
                .await
                .unwrap()
                .completion
                .unwrap()
                .send(Err("Failed to write script".to_string()))
                .unwrap();
            let response = response.await;
            assert_eq!(response.request_type, "script_generation_error");
            assert_eq!(response.additional_info, action);
            let status: serde_json::Value = serde_json::from_str(&response.json_value).unwrap();
            assert_eq!(status["status_type"], "Error");
            assert_eq!(status["message"], "Failed to write script");
        }
    }

    #[tokio::test]
    async fn unavailable_worker_returns_an_error() {
        let state = AppState::new();
        drop(state.gen_script_rx.lock().await.take());
        let response = AppState::generation_response(&state, request("open_script")).await;
        assert_eq!(response.request_type, "script_generation_error");
        assert!(response.json_value.contains("unavailable"));
    }

    #[tokio::test]
    async fn dropped_completion_returns_an_error() {
        let state = AppState::new();
        let mut jobs = state.gen_script_rx.lock().await.take().unwrap();
        let mut response = std::pin::pin!(AppState::generation_response(
            &state,
            request("save_as_generated_script")
        ));
        assert_pending(response.as_mut());
        drop(jobs.recv().await.unwrap());
        let response = response.await;
        assert_eq!(response.request_type, "script_generation_error");
        assert!(response.json_value.contains("stopped before completion"));
    }

    #[tokio::test]
    async fn save_as_requires_a_destination() {
        let state = AppState::new();
        let mut save = request("save_as_generated_script");
        save.additional_info = "  ".to_string();
        let response = AppState::generation_response(&state, save).await;
        assert_eq!(response.request_type, "script_generation_error");
        assert!(response.json_value.contains("No Save As destination"));
    }

    #[tokio::test]
    async fn save_as_writes_the_chosen_file_and_leaves_the_session_path_alone() {
        let directory =
            std::env::temp_dir().join(format!("script-save-as-target-{}", std::process::id()));
        other_fs::create_dir_all(&directory).unwrap();
        let session_output = directory.join("session.tsp");
        let chosen = directory.join("chosen.tsp");
        let state = Arc::new(AppState::new());
        let jobs = state.gen_script_rx.lock().await.take().unwrap();
        *state.work_folder.lock().await = Some(session_output.to_str().unwrap().to_string());
        let mut catalog = Catalog::new();
        catalog.refresh_function_metadata();
        let mut model = ScriptModel::new(catalog);
        model.initialize_scripts();
        model.add_sweep();
        let worker = tokio::spawn(AppState::generate_scripts(model, state.clone(), jobs));

        let mut save = request("save_as_generated_script");
        save.additional_info = chosen.to_str().unwrap().to_string();
        let response = AppState::generation_response(&state, save).await;
        assert_eq!(response.request_type, "save_as_generated_script");
        assert_eq!(
            other_fs::read_to_string(&chosen).unwrap(),
            include_str!("../../../script-gen-manager/tests/expected/no_channels.tsp")
        );
        assert!(!session_output.exists());
        assert_eq!(
            state.work_folder.lock().await.as_deref(),
            session_output.to_str()
        );

        let mut invalid = request("save_as_generated_script");
        invalid.additional_info = directory.to_str().unwrap().to_string();
        let response = AppState::generation_response(&state, invalid).await;
        assert_eq!(response.request_type, "script_generation_error");
        assert!(response.json_value.contains("Error creating script file"));

        *state.work_folder.lock().await = None;
        let mut save = request("save_as_generated_script");
        save.additional_info = chosen.to_str().unwrap().to_string();
        other_fs::write(&chosen, "stale script").unwrap();
        let response = AppState::generation_response(&state, save).await;
        assert_eq!(response.request_type, "save_as_generated_script");
        assert_eq!(
            other_fs::read_to_string(&chosen).unwrap(),
            include_str!("../../../script-gen-manager/tests/expected/no_channels.tsp")
        );
        assert!(state.work_folder.lock().await.is_none());

        worker.abort();
        assert!(worker.await.unwrap_err().is_cancelled());
        other_fs::remove_file(chosen).unwrap();
        other_fs::remove_dir(directory).unwrap();
    }

    #[tokio::test]
    async fn background_generation_reports_write_and_missing_path_errors() {
        let directory =
            std::env::temp_dir().join(format!("script-bg-worker-{}", std::process::id()));
        other_fs::create_dir_all(&directory).unwrap();
        let state = Arc::new(AppState::new());
        let jobs = state.gen_script_rx.lock().await.take().unwrap();
        *state.work_folder.lock().await = Some(directory.to_str().unwrap().to_string());
        let mut catalog = Catalog::new();
        catalog.refresh_function_metadata();
        let mut model = ScriptModel::new(catalog);
        model.initialize_scripts();
        model.add_sweep();
        let worker = tokio::spawn(AppState::generate_scripts(model, state.clone(), jobs));

        let completed = AppState::queue_background_generation(&state.gen_script_tx)
            .await
            .unwrap();
        let error = completed.await.unwrap().unwrap_err();
        assert!(error.contains("Error creating script file"));

        let error = AppState::generation_error("evaluate_data".to_string(), error);
        assert_eq!(error.request_type, "script_generation_error");
        assert_eq!(error.additional_info, "evaluate_data");

        *state.work_folder.lock().await = None;
        let completed = AppState::queue_background_generation(&state.gen_script_tx)
            .await
            .unwrap();
        assert_eq!(
            completed.await.unwrap().unwrap_err(),
            "No script output path is configured"
        );

        worker.abort();
        assert!(worker.await.unwrap_err().is_cancelled());
        other_fs::remove_dir(directory).unwrap();
    }

    #[tokio::test]
    async fn unavailable_worker_fails_background_generation() {
        let state = AppState::new();
        drop(state.gen_script_rx.lock().await.take());
        let error = AppState::queue_background_generation(&state.gen_script_tx)
            .await
            .unwrap_err();
        assert!(error.contains("unavailable"));
    }

    #[tokio::test]
    async fn worker_completes_only_after_writing_and_reports_invalid_targets() {
        let directory =
            std::env::temp_dir().join(format!("script-ack-worker-{}", std::process::id()));
        other_fs::create_dir_all(&directory).unwrap();
        let output = directory.join("script.tsp");
        other_fs::write(&output, "stale script").unwrap();
        let state = Arc::new(AppState::new());
        let jobs = state.gen_script_rx.lock().await.take().unwrap();
        *state.work_folder.lock().await = Some(output.to_str().unwrap().to_string());
        let mut catalog = Catalog::new();
        catalog.refresh_function_metadata();
        let mut model = ScriptModel::new(catalog);
        model.initialize_scripts();
        model.add_sweep();
        let worker = tokio::spawn(AppState::generate_scripts(model, state.clone(), jobs));

        let response = AppState::generation_response(&state, request("open_script")).await;
        assert_eq!(response.request_type, "open_script");
        let contents = other_fs::read_to_string(&output).unwrap();
        assert_eq!(
            contents,
            include_str!("../../../script-gen-manager/tests/expected/no_channels.tsp")
        );

        *state.work_folder.lock().await = Some(directory.to_str().unwrap().to_string());
        let response = AppState::generation_response(&state, request("open_script")).await;
        assert_eq!(response.request_type, "script_generation_error");
        assert!(response.json_value.contains("Error creating script file"));

        *state.work_folder.lock().await = None;
        let response = AppState::generation_response(&state, request("open_script")).await;
        assert_eq!(response.request_type, "script_generation_error");
        assert!(response.json_value.contains("No script output path"));

        worker.abort();
        assert!(worker.await.unwrap_err().is_cancelled());
        other_fs::remove_file(output).unwrap();
        other_fs::remove_dir(directory).unwrap();
    }
}
