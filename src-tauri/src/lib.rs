use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime, Window,
};

// 登録されたアプリケーションの情報
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegisteredApp {
    pub id: String,
    pub name: String,
    pub path: String,
    pub arguments: String,
    pub description: String,
    pub enabled: bool,
    pub delay: u64,
    #[serde(default, alias = "preventDuplicate")]
    pub prevent_duplicate: bool,
    #[serde(default, alias = "autoStart")]
    pub auto_start: bool,
}

// アプリケーション設定
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct AppConfig {
    pub registered_apps: Vec<RegisteredApp>,
}

// グローバル状態
pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub running_processes: Mutex<HashMap<String, u32>>, // app_id -> process_id
}

#[tauri::command]
fn show_window(window: Window) {
    window.show().unwrap();
}

#[tauri::command]
fn hide_window(window: Window) {
    window.hide().unwrap();
}

// 設定ファイルのパスを取得
fn get_config_path(app: &AppHandle) -> PathBuf {
    let app_dir = app
        .path()
        .app_config_dir()
        .expect("Failed to get app config dir");
    std::fs::create_dir_all(&app_dir).expect("Failed to create app config dir");
    app_dir.join("config.json")
}

// 設定ファイルを読み込み
fn load_config(app: &AppHandle) -> AppConfig {
    let config_path = get_config_path(app);
    if config_path.exists() {
        let config_str = std::fs::read_to_string(config_path).unwrap_or_default();
        serde_json::from_str(&config_str).unwrap_or_default()
    } else {
        AppConfig::default()
    }
}

// 設定ファイルを保存
fn save_config(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let config_path = get_config_path(app);
    let config_str = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(config_path, config_str).map_err(|e| e.to_string())?;
    Ok(())
}

// 登録されたアプリケーション一覧を取得
#[tauri::command]
fn get_registered_apps(app: AppHandle) -> Result<Vec<RegisteredApp>, String> {
    let state: tauri::State<AppState> = app.state();
    let config = state.config.lock().unwrap();
    Ok(config.registered_apps.clone())
}

// 設定をリセット（開発・デバッグ用）
#[tauri::command]
fn reset_config(app: AppHandle) -> Result<(), String> {
    let state: tauri::State<AppState> = app.state();
    let mut config = state.config.lock().unwrap();

    // 設定をクリア
    config.registered_apps.clear();

    // 設定ファイルを保存
    save_config(&app, &config)?;

    println!("Configuration has been reset");
    Ok(())
}

// アプリケーションを登録
#[tauri::command]
fn add_registered_app(
    app: AppHandle,
    name: String,
    path: String,
    arguments: String,
    description: String,
    enabled: bool,
    delay: u64,
    prevent_duplicate: bool,
    auto_start: bool,
) -> Result<RegisteredApp, String> {
    let state: tauri::State<AppState> = app.state();
    let mut config = state.config.lock().unwrap();

    let new_app = RegisteredApp {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        path,
        arguments,
        description,
        enabled,
        delay,
        prevent_duplicate,
        auto_start,
    };

    config.registered_apps.push(new_app.clone());
    save_config(&app, &config)?;

    Ok(new_app)
}

// アプリケーション情報を更新
#[tauri::command]
fn update_registered_app(
    app: AppHandle,
    id: String,
    name: String,
    path: String,
    arguments: String,
    description: String,
    enabled: bool,
    delay: u64,
    prevent_duplicate: bool,
    auto_start: bool,
) -> Result<(), String> {
    let state: tauri::State<AppState> = app.state();
    let mut config = state.config.lock().unwrap();

    if let Some(app_entry) = config.registered_apps.iter_mut().find(|a| a.id == id) {
        app_entry.name = name;
        app_entry.path = path;
        app_entry.arguments = arguments;
        app_entry.description = description;
        app_entry.enabled = enabled;
        app_entry.delay = delay;
        app_entry.prevent_duplicate = prevent_duplicate;
        app_entry.auto_start = auto_start;

        save_config(&app, &config)?;
        Ok(())
    } else {
        Err("Application not found".to_string())
    }
}

// アプリケーションを削除
#[tauri::command]
fn remove_registered_app(app: AppHandle, id: String) -> Result<(), String> {
    let state: tauri::State<AppState> = app.state();
    let mut config = state.config.lock().unwrap();

    config.registered_apps.retain(|a| a.id != id);
    save_config(&app, &config)?;

    Ok(())
}

// アプリケーションを起動
#[tauri::command]
async fn launch_application(
    app: AppHandle,
    app_id: String,
    path: String,
    arguments: String,
) -> Result<(), String> {
    // 登録されたアプリケーションの情報を確認
    let state: tauri::State<AppState> = app.state();
    let config = state.config.lock().unwrap();
    let registered_app = config.registered_apps.iter().find(|app| app.id == app_id);
    let is_registered_app = registered_app.is_some();
    let prevent_duplicate = registered_app
        .map(|app| app.prevent_duplicate)
        .unwrap_or(false);
    drop(config);

    if is_registered_app {
        // 登録されたアプリケーションの場合
        #[cfg(target_os = "windows")]
        {
            if prevent_duplicate {
                // 重複起動禁止の場合はプロセスIDを取得せずシンプルに起動
                let quoted_path = format!("'{}'", path);
                let mut powershell_command = format!("Start-Process -FilePath {}", quoted_path);

                if !arguments.trim().is_empty() {
                    let quoted_args = format!("'{}'", arguments);
                    powershell_command = format!(
                        "Start-Process -FilePath {} -ArgumentList {}",
                        quoted_path, quoted_args
                    );
                }

                println!(
                    "Executing simple launch command (prevent_duplicate): {}",
                    powershell_command
                );

                let output = Command::new("powershell")
                    .args(&["-WindowStyle", "Hidden", "-Command", &powershell_command])
                    .output()
                    .map_err(|e| format!("Failed to launch application: {}", e))?;

                if output.status.success() {
                    println!(
                        "Application launched successfully (prevent_duplicate, no PID tracking)"
                    );

                    // プロセス名ベース管理のマーカーを記録
                    let mut processes = state.running_processes.lock().unwrap();
                    processes.insert(format!("{}:name", app_id), 0);
                    println!(
                        "Stored process name tracking for app_id: {} (prevent_duplicate)",
                        app_id
                    );

                    return Ok(());
                } else {
                    let error_msg = String::from_utf8_lossy(&output.stderr);
                    return Err(format!("Start-Process failed: {}", error_msg));
                }
            } else {
                // 通常の場合はプロセスIDを取得
                let quoted_path = format!("'{}'", path);
                let mut powershell_command = format!(
                    "$process = Start-Process -FilePath {} -PassThru",
                    quoted_path
                );

                if !arguments.trim().is_empty() {
                    let quoted_args = format!("'{}'", arguments);
                    powershell_command = format!(
                        "$process = Start-Process -FilePath {} -ArgumentList {} -PassThru",
                        quoted_path, quoted_args
                    );
                }

                powershell_command.push_str("; Write-Output $process.Id");

                println!(
                    "Executing PID tracking launch command: {}",
                    powershell_command
                );

                let output = Command::new("powershell")
                    .args(&["-WindowStyle", "Hidden", "-Command", &powershell_command])
                    .output()
                    .map_err(|e| {
                        format!("Failed to launch application with Start-Process: {}", e)
                    })?;

                if output.status.success() {
                    let pid_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if let Ok(actual_pid) = pid_str.parse::<u32>() {
                        println!("Started application with PID: {}", actual_pid);

                        let mut processes = state.running_processes.lock().unwrap();
                        processes.insert(app_id.clone(), actual_pid);
                        println!("Stored PID {} for app_id: {}", actual_pid, app_id);

                        return Ok(());
                    } else {
                        return Err(format!("Failed to parse process ID: {}", pid_str));
                    }
                } else {
                    let error_msg = String::from_utf8_lossy(&output.stderr);
                    return Err(format!("Start-Process failed: {}", error_msg));
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Windows以外では従来通り
            let mut cmd = Command::new(&path);
            if !arguments.trim().is_empty() {
                let args: Vec<&str> = arguments.split_whitespace().collect();
                cmd.args(&args);
            }
            let child = cmd
                .spawn()
                .map_err(|e| format!("Failed to launch application: {}", e))?;

            // プロセスIDを記録
            let mut processes = state.running_processes.lock().unwrap();
            processes.insert(app_id, child.id());
            return Ok(());
        }
    } else {
        // システムツールの場合は従来通り
        let mut cmd = Command::new(&path);
        if !arguments.trim().is_empty() {
            let args: Vec<&str> = arguments.split_whitespace().collect();
            cmd.args(&args);
        }
        let child = cmd
            .spawn()
            .map_err(|e| format!("Failed to launch application: {}", e))?;

        // プロセスIDを記録
        let mut processes = state.running_processes.lock().unwrap();
        processes.insert(app_id, child.id());
        return Ok(());
    }
}

// アプリケーションを停止
#[tauri::command]
fn stop_application(app: AppHandle, app_id: String) -> Result<(), String> {
    let state: tauri::State<AppState> = app.state();

    // 登録されたアプリケーションの情報を取得
    let config = state.config.lock().unwrap();
    let registered_app = config.registered_apps.iter().find(|app| app.id == app_id);
    let prevent_duplicate = registered_app
        .map(|app| app.prevent_duplicate)
        .unwrap_or(false);
    let app_name = registered_app.map(|app| app.name.clone());
    drop(config);

    // プロセス管理テーブルから確認
    let mut processes = state.running_processes.lock().unwrap();

    // 重複起動禁止の場合は特別なキーで確認
    let process_key = if prevent_duplicate {
        format!("{}:name", app_id)
    } else {
        app_id.clone()
    };

    let pid = processes.get(&process_key).copied();

    if let Some(pid) = pid {
        processes.remove(&process_key);
        drop(processes);

        if prevent_duplicate {
            // 重複起動禁止の場合はアプリ名で停止
            if let Some(process_name) = app_name {
                println!(
                    "Attempting to stop process by name: {} for app: {} (prevent_duplicate)",
                    process_name, app_id
                );

                #[cfg(target_os = "windows")]
                {
                    let output = Command::new("powershell")
                        .args(&[
                            "-WindowStyle",
                            "Hidden",
                            "-Command",
                            &format!("Stop-Process -Name '{}' -Force", process_name),
                        ])
                        .output();

                    return match output {
                        Ok(result) => {
                            if result.status.success() {
                                println!("Successfully stopped process by name: {}", process_name);
                                Ok(())
                            } else {
                                let error_msg = String::from_utf8_lossy(&result.stderr);
                                println!("Stop-Process by name failed: {}", error_msg);
                                Err(format!(
                                    "Failed to stop process '{}': {}",
                                    process_name, error_msg
                                ))
                            }
                        }
                        Err(e) => {
                            println!("Failed to execute Stop-Process by name: {}", e);
                            Err(format!(
                                "Failed to stop application with Stop-Process: {}",
                                e
                            ))
                        }
                    };
                }

                #[cfg(not(target_os = "windows"))]
                {
                    return Err(
                        "Process name based termination not supported on this platform".to_string(),
                    );
                }
            } else {
                return Err("Application path not found".to_string());
            }
        } else {
            // 通常のアプリの場合はPIDで停止
            println!("Attempting to stop process ID: {} for app: {}", pid, app_id);

            #[cfg(target_os = "windows")]
            {
                let output = Command::new("powershell")
                    .args(&[
                        "-WindowStyle",
                        "Hidden",
                        "-Command",
                        &format!("Stop-Process -Id {} -Force", pid),
                    ])
                    .output();

                return match output {
                    Ok(result) => {
                        if result.status.success() {
                            println!("Successfully stopped process {}", pid);
                            Ok(())
                        } else {
                            let error_msg = String::from_utf8_lossy(&result.stderr);
                            println!("Stop-Process failed: {}", error_msg);
                            Err(format!("Failed to stop process {}: {}", pid, error_msg))
                        }
                    }
                    Err(e) => {
                        println!("Failed to execute Stop-Process: {}", e);
                        Err(format!(
                            "Failed to stop application with Stop-Process: {}",
                            e
                        ))
                    }
                };
            }

            #[cfg(not(target_os = "windows"))]
            {
                let output = Command::new("kill")
                    .args(&["-9", &pid.to_string()])
                    .output();

                return match output {
                    Ok(_) => Ok(()),
                    Err(e) => Err(format!("Failed to stop application: {}", e)),
                };
            }
        }
    }

    Err("Application not found or not running".to_string())
}

// アプリケーションの実行状態を確認
#[tauri::command]
fn is_application_running(app: AppHandle, app_id: String) -> bool {
    let state: tauri::State<AppState> = app.state();
    let processes = state.running_processes.lock().unwrap();
    processes.contains_key(&app_id)
}

// 登録された全アプリケーションを起動（自動起動用）
#[tauri::command]
async fn launch_startup_apps(app: AppHandle) -> Result<(), String> {
    let state: tauri::State<AppState> = app.state();
    let config = state.config.lock().unwrap().clone();

    for registered_app in config.registered_apps.iter().filter(|a| a.enabled) {
        let app_id = registered_app.id.clone();
        let path = registered_app.path.clone();
        let arguments = registered_app.arguments.clone();
        let delay = registered_app.delay;
        let prevent_duplicate = registered_app.prevent_duplicate;
        let app_handle_clone = app.clone();

        // 重複起動禁止が有効な場合、既存プロセスを停止
        if prevent_duplicate {
            let process_name = registered_app.name.clone();

            println!("Preventing duplicate launch for: {}", process_name);

            #[cfg(target_os = "windows")]
            {
                use std::process::Command;
                let _output = Command::new("powershell")
                    .args(&[
                        "-WindowStyle",
                        "Hidden",
                        "-Command",
                        &format!(
                            "Stop-Process -Name '{}' -Force -ErrorAction SilentlyContinue",
                            process_name
                        ),
                    ])
                    .output();
                // エラーは無視（プロセスが存在しない場合もあるため）
            }
        }

        // 遅延がある場合は待機
        if delay > 0 {
            tokio::time::sleep(Duration::from_secs(delay)).await;
        }

        // アプリケーションを起動
        let result = launch_application(app_handle_clone, app_id, path, arguments).await;
        if let Err(e) = result {
            eprintln!("Failed to launch {}: {}", registered_app.name, e);
        }
    }

    Ok(())
}

// ファイル選択ダイアログを開く
#[tauri::command]
fn open_file_dialog(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let file_path = app
        .dialog()
        .file()
        .add_filter("実行ファイル", &["exe"])
        .add_filter("ショートカット", &["lnk"])
        .add_filter("すべてのファイル", &["*"])
        .blocking_pick_file();

    Ok(file_path.map(|p| p.to_string()))
}

fn create_tray_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let show_item = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let hide_item = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    Menu::with_items(app, &[&show_item, &hide_item, &quit_item])
}

fn handle_tray_event(app: &AppHandle, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: tauri::tray::MouseButtonState::Up,
        ..
    } = event
    {
        let window = app.get_webview_window("main").unwrap();
        let _ = window.show();
    }
}

fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    let window = app.get_webview_window("main").unwrap();

    match event.id.as_ref() {
        "show" => {
            let _ = window.show();
        }
        "hide" => {
            let _ = window.hide();
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // アプリケーション状態を初期化
            let config = load_config(app.handle());
            app.manage(AppState {
                config: Mutex::new(config),
                running_processes: Mutex::new(HashMap::new()),
            });

            let menu = create_tray_menu(app.handle())?;

            let _tray = TrayIconBuilder::with_id("main")
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Ajiponzu Utility Launcher")
                .on_menu_event(|app, event| handle_menu_event(app, event))
                .on_tray_icon_event(|tray, event| {
                    let app = tray.app_handle();
                    handle_tray_event(app, event);
                })
                .build(app)?;

            // アプリケーション起動時に自動起動を実行
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = launch_startup_apps(app_handle).await {
                    eprintln!("Failed to launch startup apps: {}", e);
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // アプリを終了させずにウィンドウを隠す
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            show_window,
            hide_window,
            get_registered_apps,
            add_registered_app,
            update_registered_app,
            remove_registered_app,
            reset_config,
            launch_application,
            stop_application,
            is_application_running,
            launch_startup_apps,
            open_file_dialog
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
