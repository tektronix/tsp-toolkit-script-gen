use actix_files as fs;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_ws::{Message, Session};
use futures::StreamExt;
use script_gen_manager::script_component::script::ScriptModel;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

use tokio::io::{self, AsyncBufReadExt};
use tokio::signal;
use tokio::sync::watch;

use crate::back_end::ipc_data::IpcData;

use super::data_model::DataModel;

#[derive(Clone)]
pub struct AppState {
    pub session: Arc<Mutex<Option<Session>>>,
    pub data_model: Arc<Mutex<DataModel>>,
    pub gen_script_tx: broadcast::Sender<()>,
}

impl AppState {
    pub fn new() -> Self {
        let (gen_script_tx, _) = broadcast::channel(100); // Create a broadcast channel
        AppState {
            session: Arc::new(Mutex::new(None)),
            data_model: Arc::new(Mutex::new(DataModel::new())),
            gen_script_tx,
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
                    println!("Received input from client in session: {}", msg);
                    match serde_json::from_str::<IpcData>(&msg) {
                        Ok(ipc_data) => {
                            if ipc_data.request_type == "get_data" {
                                println!("instrument data requested");
                            } else if ipc_data.request_type == "evaluate_data" {
                                let mut data_model = app_state.data_model.lock().await;
                                let response =
                                    data_model.process_data_from_client(ipc_data.json_value);
                                println!("{}", response);
                                // Send generate script signal
                                if let Err(e) = gen_script_tx.send(()) {
                                    eprintln!("Failed to send signal: {}", e);
                                }
                                session.text(response).await.unwrap();
                            } else if ipc_data.request_type == "reallocation" {
                                let mut data_model = app_state.data_model.lock().await;
                                let response = data_model.add_remove_channel(ipc_data);
                                println!("{}", response);
                                // Send generate script signal
                                if let Err(e) = gen_script_tx.send(()) {
                                    eprintln!("Failed to send signal: {}", e);
                                }
                                session.text(response).await.unwrap();
                            } else {
                                println!("Unknown request type: {}", ipc_data.request_type);
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to deserialize IpcData: {}", e);
                        }
                    }
                }
                Message::Close(reason) => {
                    println!("Connection closed: {:?}", reason);
                    return;
                }
                _ => (),
            }
        }
    });

    Ok(response)
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
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .route(
                "/",
                web::get().to(|| async {
                    let exe_path = std::env::current_exe()?;
                    let Some(exe_dir) = exe_path.parent() else {
                        return Err(std::io::Error::other(
                            "Unable to get directory of server executable",
                        ));
                    };
                    fs::NamedFile::open(format!("{}/browser/index.html", exe_dir.display()))
                }),
            )
            .route("/ws", web::get().to(ws_index))
            .service(
                fs::Files::new("/", format!("{}/browser", exe_dir.display()))
                    .index_file("index.html"),
            )
            .wrap(
                actix_cors::Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec!["Content-Type"]),
            )
    })
    .bind(("127.0.0.1", 8080))?
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
                if let Some(sweep_model) = &data_model.sweep {
                    let sweep_config = &sweep_model.sweep_config;
                    script_model.to_script(sweep_config);
                } else {
                    println!("No SweepModel found in data_model.sweep");
                }
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
        //let gen_script_tx = app_state.gen_script_tx.clone();

        while let Some(line) = reader.next_line().await.unwrap() {
            println!("Received from stdin: {line}");
            if line.trim() == "shutdown" {
                println!("Received shutdown command from stdin, shutting down...");
                let _ = shutdown_tx.send(());
                break;
            } else if line.trim() == "reload" {
                println!("Received reload command from stdin, reloading...");
                let json_str = r#"
                    [
                        {
                            "name": "localnode",
                            "model": "MP5103",
                            "slot":
                            [
                                {
                                    "name": "slot[1]",
                                    "model": "MPSU50-2ST"
                                },
                                {
                                    "name": "slot[2]",
                                    "model": "MSMU60-2"
                                },
                                {
                                    "name": "slot[3]",
                                    "model": "MSMU60-2"
                                }
                            ]
                        }
                    ]
                "#;

                let mut data_model = app_state.data_model.lock().await;
                let response = data_model.process_instr_info(json_str.to_string());
                println!("{}", response);
                // Send generate script signal
                if let Err(e) = app_state.gen_script_tx.send(()) {
                    eprintln!("Failed to send signal: {}", e);
                }
                let mut session = app_state.session.lock().await;
                if let Some(session) = session.as_mut() {
                    session.text(response).await.unwrap();
                }
            }
        }
    });

    //run()?;
    server.await?;
    Ok(())
}
