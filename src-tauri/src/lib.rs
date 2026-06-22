mod commands;
mod db;
mod image;
mod models;

use commands::{
    add_family_member, add_person, add_relationship, delete_person, delete_relationship,
    get_family, get_graph, list_people, list_relationships, remove_family_member, update_person,
    update_relationship,
};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let pool = db::init_pool().await;
                handle.manage(pool);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_people,
            add_person,
            update_person,
            delete_person,
            list_relationships,
            add_relationship,
            update_relationship,
            delete_relationship,
            get_graph,
            get_family,
            add_family_member,
            remove_family_member,
        ])
        .run(tauri::generate_context!())
        .expect("Fehler beim Starten der Tauri-App");
}
