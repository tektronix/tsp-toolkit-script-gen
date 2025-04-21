use actix_files as fs;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_ws::{Message, Session};
use futures::StreamExt;
use script_gen_manager::catalog::Catalog;
use std::sync::Arc;
use tokio::sync::Mutex;

use tokio::io::{self, AsyncBufReadExt};
use tokio::signal;
use tokio::sync::watch;

use crate::back_end::ipc_data::IpcData;

use super::data_model::DataModel;

#[derive(Clone)]
pub struct AppState {
    pub session: Arc<Mutex<Option<Session>>>,
    pub data_model: Arc<Mutex<DataModel>>,
}

impl AppState {
    pub fn new(catalog: Catalog) -> Self {
        AppState {
            session: Arc::new(Mutex::new(None)),
            data_model: Arc::new(Mutex::new(DataModel::new(catalog))),
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
                                session.text(response).await.unwrap();
                            } else if ipc_data.request_type == "reallocation" {
                                let mut data_model = app_state.data_model.lock().await;
                                let response = data_model.add_remove_channel(ipc_data);
                                println!("{}", response);
                                session.text(response).await.unwrap();
                            } else {
                                println!("Unknown request type: {}", ipc_data.request_type);
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to deserialize IpcData: {}", e);
                        }
                    }
                    // let mut data_model = app_state.data_model.lock().await;
                    // let response = data_model.process_data(msg.to_string());
                    // session.text(response).await.unwrap();
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
        App::new()
        .app_data(web::Data::new(app_state.clone()))
            .route("/", web::get().to(|| async {
                fs::NamedFile::open("C:\\git\\TekSpecific\\tsp-toolkit-script-gen\\script-gen-ui\\dist\\script-gen-ui\\browser\\index.html")
            }))
            .route("/ws", web::get().to(ws_index))
            .service(fs::Files::new("/", "C:\\git\\TekSpecific\\tsp-toolkit-script-gen\\script-gen-ui\\dist\\script-gen-ui\\browser").index_file("index.html"))            .wrap(
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

pub async fn start(catalog: Catalog) -> anyhow::Result<()> {
    let app_state = Arc::new(AppState::new(catalog));

    let (shutdown_tx, shutdown_rx) = watch::channel(());
    let server = start_web_server(app_state.clone(), shutdown_rx.clone());

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
