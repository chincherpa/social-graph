// Entry point. Die eigentliche Logik liegt in lib.rs (Tauri 2.x Konvention),
// damit der gleiche Code auch für mobile_entry_point genutzt werden kann.
fn main() {
    social_graph_lib::run();
}
