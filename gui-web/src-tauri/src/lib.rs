mod bridge;
mod commands;
mod error;
mod events;

use bridge::AppBridge;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let log_level = if cfg!(debug_assertions) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log_level)
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: None,
                    }),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
                ])
                .build(),
        )
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(AppBridge::new())
        .invoke_handler(tauri::generate_handler![
            commands::connect,
            commands::connect_futopt,
            commands::disconnect,
            commands::subscribe,
            commands::unsubscribe,
            commands::subscribe_futopt,
            commands::unsubscribe_futopt,
            commands::fetch_candles,
            commands::fetch_ticker,
            commands::fetch_trades,
            commands::fetch_quote,
            commands::fetch_futopt_ticker,
            commands::fetch_futopt_quote,
            commands::fetch_futopt_trades,
            commands::fetch_futopt_candles,
            commands::fetch_futopt_products,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
