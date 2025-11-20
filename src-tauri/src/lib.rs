mod db;
mod connection;
mod metadata;
mod cache;
mod commands;
mod yaml_import;

use db::init_db;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize database
    if let Err(e) = init_db() {
        eprintln!("Failed to initialize database: {}", e);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // Context commands
            commands::create_context,
            commands::list_contexts,
            commands::update_context,
            commands::delete_context,
            // DataSource commands
            commands::create_data_source,
            commands::list_data_sources,
            commands::get_data_source,
            commands::update_data_source,
            commands::delete_data_source,
            commands::test_connection,
            // Metadata commands
            commands::get_tables,
            commands::get_table_structure,
            commands::get_kafka_topics,
            commands::get_schema_registry_schemas,
            commands::refresh_metadata,
            // Comparison command
            commands::compare_tables,
            // YAML import
            yaml_import::import_data_sources_from_yaml,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
