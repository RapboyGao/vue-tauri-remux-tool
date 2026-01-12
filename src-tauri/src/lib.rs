// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::process::{Command, Stdio};
use std::path::Path;
use std::fs;
use std::io::{BufRead, BufReader};
use std::sync::Mutex;
use tauri::Emitter;
use tauri::async_runtime::spawn;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn check_ffmpeg_version(ffmpeg_path: &str) -> Result<String, String> {
    println!("Checking FFmpeg version at path: {}", ffmpeg_path);
    
    // 检查文件是否存在
    if !Path::new(ffmpeg_path).exists() {
        return Err(format!("FFmpeg file not found at path: {}", ffmpeg_path));
    }
    
    // 检查文件是否可执行
    if !Path::new(ffmpeg_path).is_file() {
        return Err(format!("Path is not a file: {}", ffmpeg_path));
    }
    
    let output = Command::new(ffmpeg_path)
        .arg("-version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to execute FFmpeg: {}", e))?;
    
    println!("FFmpeg execution exit code: {}", output.status);
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFmpeg execution failed: {}", stderr));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let version_line = stdout.lines().next().unwrap_or("");
    
    if version_line.is_empty() {
        return Err("Failed to parse FFmpeg version from output".to_string());
    }
    
    println!("FFmpeg version detected: {}", version_line);
    Ok(version_line.to_string())
}

// 添加下载相关的导入
use reqwest::blocking::get;

#[tauri::command]
fn update_ffmpeg(ffmpeg_path: &str) -> Result<bool, String> {
    println!("Updating FFmpeg to latest version...");
    println!("Target path: {}", ffmpeg_path);
    
    // 创建FFmpeg目录
    let ffmpeg_dir = Path::new(ffmpeg_path).parent()
        .ok_or_else(|| "Invalid FFmpeg path".to_string())?;
    if !ffmpeg_dir.exists() {
        fs::create_dir_all(ffmpeg_dir)
            .map_err(|e| format!("Failed to create FFmpeg directory: {}", e))?;
    }
    
    // 确定平台和架构（使用npmmirror.com的命名约定）
    let (platform, arch) = {
        let arch = if cfg!(target_arch = "x86_64") {
            "x64"
        } else if cfg!(target_arch = "aarch64") {
            "arm64"
        } else if cfg!(target_arch = "x86") {
            "ia32"
        } else {
            return Err(format!("Unsupported architecture: {}", std::env::consts::ARCH));
        };
        
        if cfg!(target_os = "macos") {
            ("darwin", arch)
        } else if cfg!(target_os = "windows") {
            ("win32", arch)
        } else if cfg!(target_os = "linux") {
            ("linux", arch)
        } else {
            return Err(format!("Unsupported platform: {}", std::env::consts::OS));
        }
    };
    
    // 构建npmmirror.com的下载URL
    let download_url = format!("https://registry.npmmirror.com/-/binary/ffmpeg-static/b6.1.1/ffmpeg-{}-{}", platform, arch);
    println!("Downloading FFmpeg from: {}", download_url);
    
    // 下载FFmpeg可执行文件
    let response = get(&download_url)
        .map_err(|e| format!("Failed to download FFmpeg: {}", e))?;
    
    println!("Download completed, status: {}", response.status());
    
    if !response.status().is_success() {
        return Err(format!("Failed to download FFmpeg: HTTP {}", response.status()));
    }
    
    // 读取响应体
    let content = response.bytes()
        .map_err(|e| format!("Failed to read response: {}", e))?;
    
    // 将下载的内容直接写入目标文件
    fs::write(ffmpeg_path, content)
        .map_err(|e| format!("Failed to write FFmpeg executable: {}", e))?;
    
    // 设置可执行权限（对于非Windows平台）
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(ffmpeg_path)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?.permissions();
        perms.set_mode(0o755); // rwxr-xr-x
        fs::set_permissions(ffmpeg_path, perms)
            .map_err(|e| format!("Failed to set executable permissions: {}", e))?;
    }
    
    println!("Successfully updated FFmpeg to: {}", ffmpeg_path);
    
    Ok(true)
}

#[tauri::command]
fn detect_bdmv_structure(path: &str) -> bool {
    // 检查是否包含BDMV目录
    let bdmv_path = Path::new(path).join("BDMV");
    bdmv_path.exists()
}

#[tauri::command]
fn get_media_info(ffmpeg_path: &str, input_path: &str) -> Result<String, String> {
    let output = Command::new(ffmpeg_path)
        .args(["-i", input_path, "-hide_banner"])
        .output()
        .map_err(|e| format!("Failed to execute FFmpeg: {}", e))?;
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    Ok(stderr.to_string())
}

// 存储FFmpeg进程ID的全局变量
lazy_static::lazy_static! {
    static ref FFMPEG_PROCESSES: Mutex<std::collections::HashMap<i32, std::process::Child>> = Mutex::new(std::collections::HashMap::new());
}

#[tauri::command]
async fn run_ffmpeg(ffmpeg_path: &str, args: Vec<&str>, window: tauri::Window) -> Result<i32, String> {
    // 启动FFmpeg进程，捕获标准输出和标准错误
    let mut child = Command::new(ffmpeg_path)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to execute FFmpeg: {}", e))?;
    
    let pid = child.id();
    let stdout = child.stdout.take()
        .ok_or_else(|| "Failed to capture stdout".to_string())?;
    let stderr = child.stderr.take()
        .ok_or_else(|| "Failed to capture stderr".to_string())?;
    
    // 将u32 pid转换为i32
    let pid_i32 = pid.try_into()
        .map_err(|e| format!("Failed to convert pid: {}", e))?;
    
    // 保存进程ID
    FFMPEG_PROCESSES.lock().unwrap().insert(pid_i32, child);
    
    // 为每个异步任务创建window的克隆
    let window_stdout = window.clone();
    let window_stderr = window.clone();
    
    // 读取标准输出
    spawn(async move {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                // 使用result处理emit可能的失败，避免程序崩溃
                let _ = window_stdout.emit("ffmpeg-output", format!("stdout: {}", line));
                
                // 解析进度信息
                if line.contains("time=") && line.contains("bitrate=") {
                    let _ = window_stdout.emit("ffmpeg-progress", line);
                }
            }
        }
    });
    
    // 读取标准错误
    spawn(async move {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(line) = line {
                // 使用result处理emit可能的失败，避免程序崩溃
                let _ = window_stderr.emit("ffmpeg-output", format!("stderr: {}", line));
                
                // 解析进度信息
                if line.contains("time=") && line.contains("bitrate=") {
                    let _ = window_stderr.emit("ffmpeg-progress", line);
                }
            }
        }
    });
    
    // 返回进程ID
    Ok(pid_i32)
}

#[allow(dead_code)]
#[tauri::command]
async fn get_ffmpeg_output(pid: i32) -> Result<String, String> {
    // 这里可以实现获取特定进程的输出
    Ok(format!("Getting output for process {}", pid))
}

#[tauri::command]
async fn stop_ffmpeg(pid: i32) -> Result<bool, String> {
    println!("Stopping FFmpeg process with PID: {}", pid);
    
    let mut processes = FFMPEG_PROCESSES.lock().unwrap();
    if let Some(mut child) = processes.remove(&pid) {
        // 发送终止信号
        child.kill()
            .map_err(|e| format!("Failed to kill FFmpeg process: {}", e))?;
        // 等待进程退出
        let _ = child.wait();
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
async fn read_dir(path: &str) -> Result<Vec<serde_json::Value>, String> {
    let mut entries = Vec::new();
    
    for entry in std::fs::read_dir(path)
        .map_err(|e| format!("Failed to read directory: {}", e))? {
        if let Ok(entry) = entry {
            let path = entry.path();
            let name = path.file_name()
                .map(|os_str| os_str.to_string_lossy().to_string())
                .unwrap_or("unnamed".to_string());
            let is_directory = path.is_dir();
            
            entries.push(serde_json::json!({
                "name": name,
                "path": path.to_string_lossy().to_string(),
                "isDirectory": is_directory
            }));
        }
    }
    
    Ok(entries)
}

#[tauri::command]
async fn file_exists(path: &str) -> Result<bool, String> {
    Ok(std::path::Path::new(path).exists())
}

#[tauri::command]
async fn select_directory(_title: &str) -> Result<String, String> {
    // 这里应该实现实际的目录选择对话框
    // 示例：返回当前目录
    Ok(".".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            check_ffmpeg_version,
            update_ffmpeg,
            detect_bdmv_structure,
            get_media_info,
            run_ffmpeg,
            stop_ffmpeg,
            read_dir,
            file_exists,
            select_directory
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
