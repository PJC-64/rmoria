use tauri::Emitter;

struct InputSender(std::sync::Mutex<std::sync::mpsc::Sender<crossterm::event::Event>>);
struct SetupSender(std::sync::Mutex<std::sync::mpsc::Sender<serde_json::Value>>);

#[tauri::command]
fn gui_send_key(key: String, state: tauri::State<'_, InputSender>) {
    let tx = state.0.lock().unwrap();
    let event = match key.as_str() {
        "\x1b" => crossterm::event::Event::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Esc,
            crossterm::event::KeyModifiers::empty(),
        )),
        "\n" => crossterm::event::Event::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::empty(),
        )),
        "\u{0008}" => crossterm::event::Event::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Backspace,
            crossterm::event::KeyModifiers::empty(),
        )),
        s if s.len() == 1 => {
            let c = s.chars().next().unwrap();
            crossterm::event::Event::Key(crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(c),
                crossterm::event::KeyModifiers::empty(),
            ))
        }
        _ => return,
    };
    let _ = tx.send(event);
}

#[tauri::command]
fn gui_send_setup(action: serde_json::Value, state: tauri::State<'_, SetupSender>) {
    let tx = state.0.lock().unwrap();
    let _ = tx.send(action);
}

#[tauri::command]
fn gui_check_save() -> bool {
    rmoria::get_save_path().exists()
}

#[tauri::command]
fn gui_activate_cheat(state: tauri::State<'_, InputSender>) {
    rmoria::activate_cheat_global();
    let tx = state.0.lock().unwrap();
    let dummy = crossterm::event::Event::Key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char(' '),
        crossterm::event::KeyModifiers::empty(),
    ));
    let _ = tx.send(dummy);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (input_tx, state_rx, setup_tx) = rmoria::init_gui_mode();

    tauri::Builder::default()
        .manage(InputSender(std::sync::Mutex::new(input_tx)))
        .manage(SetupSender(std::sync::Mutex::new(setup_tx)))
        .invoke_handler(tauri::generate_handler![gui_send_key, gui_send_setup, gui_check_save, gui_activate_cheat])
        .setup(move |app| {
            if cfg!(debug_assertions) {
                let _ = app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                );
            }

            let app_handle = app.handle().clone();
            let app_handle_for_game = app_handle.clone();

            // Spawn Moria game loop in a background thread
            std::thread::spawn(move || {
                let _ = rmoria::run_cli();
                app_handle_for_game.exit(0);
            });

            // Spawn state receiver thread to emit updates to frontend
            std::thread::spawn(move || {
                while let Ok(gui_state) = state_rx.recv() {
                    let payload = serde_json::json!({
                        "screen": gui_state.screen,
                        "status_msg": gui_state.status_msg,
                        "player": gui_state.player_json
                    });
                    let _ = app_handle.emit("gui-update", payload);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
