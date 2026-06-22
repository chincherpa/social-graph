mod commands;
mod db;
mod image;
mod models;

use commands::{
    add_family_member, add_person, add_relationship, delete_person, delete_person_image,
    delete_relationship, geocode_person, get_family, get_graph, list_people, list_relationships,
    remove_family_member, swap_relationship_direction, update_person, update_relationship,
    upload_person_image,
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
            swap_relationship_direction,
            get_graph,
            get_family,
            add_family_member,
            remove_family_member,
            upload_person_image,
            delete_person_image,
            geocode_person,
        ])
        .run(tauri::generate_context!())
        .expect("Fehler beim Starten der Tauri-App");
}
