use std::process::Command;

#[tauri::command]
fn run_engcode(code: String) -> Result<String, String> {
    // Write code to a temporary file
    let temp_file = "/tmp/engcode_temp.eng";
    std::fs::write(temp_file, &code).map_err(|e| e.to_string())?;

    // Execute engcode binary
    let output = Command::new("../target/release/engcode")
        .arg("run")
        .arg(temp_file)
        .output()
        .map_err(|e| format!("Failed to execute engcode: {}", e))?;

    // Combine stdout and stderr
    let mut result = String::from_utf8_lossy(&output.stdout).to_string();
    if !output.stderr.is_empty() {
        result.push_str("\n");
        result.push_str(&String::from_utf8_lossy(&output.stderr));
    }

    // Clean up temp file
    let _ = std::fs::remove_file(temp_file);

    Ok(result)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![run_engcode])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
