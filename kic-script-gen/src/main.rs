use kic_script_gen::back_end::client_server::start;
use script_gen_manager::{catalog, script_component::script::ScriptModel};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    eprintln!("Welcome to KIC Script Generator!");

    let mut catalog = catalog::Catalog::new();
    catalog.refresh_function_metadata();

    let mut script_model = ScriptModel::new(catalog);
    script_model.initialize_scripts();
    script_model.add_sweep();
    script_model.add_data_report();

    start(script_model).await?;

    Ok(())
}
