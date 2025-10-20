use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use bcrypt::{hash, verify, DEFAULT_COST};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct User {
    username: String,
    password_hash: String,
}

#[derive(Default)]
struct AppState {
    users: Mutex<HashMap<String, User>>,
    sessions: Mutex<HashMap<String, String>>,

fn hash_password(password: &str) -> Result<String, String> {
    hash(password, DEFAULT_COST).map_err(|e| e.to_string())
}

fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    verify(password, hash).map_err(|e| e.to_string())
}

#[tauri::command]
async fn register(
    username: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut users = state.users.lock().unwrap();

    if users.contains_key(&username) {
        return Err("Utilisateur déjà existant".to_string());
    }

    let password_hash = hash_password(&password)?;
    let user = User {
        username: username.clone(),
        password_hash,
    };

    users.insert(username, user);
    Ok(())
}

#[tauri::command]
async fn login(
    username: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let users = state.users.lock().unwrap();

    match users.get(&username) {
        Some(user) => {
            if verify_password(&password, &user.password_hash)? {
                let session_id = format!("{}-session", username);
                let mut sessions = state.sessions.lock().unwrap();
                sessions.insert(session_id.clone(), username);
                Ok(session_id)
            } else {
                Err("Mot de passe incorrect".to_string())
            }
        }
        None => Err("Utilisateur non trouvé".to_string()),
    }
}

#[tauri::command]
async fn check_session(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let sessions = state.sessions.lock().unwrap();
    Ok(sessions.contains_key(&session_id))
}

#[tauri::command]
async fn logout(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut sessions = state.sessions.lock().unwrap();
    sessions.remove(&session_id);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .manage(AppState::default())
    .invoke_handler(tauri::generate_handler![
        register,
        login,
        check_session,
        logout
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
