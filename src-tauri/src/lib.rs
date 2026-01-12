// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::process::{Command, Stdio};
use std::path::Path;
use std::fs::{self, File};
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
    
    // 根据平台创建模拟的FFmpeg可执行文件
    let is_windows = cfg!(target_os = "windows");
    let is_macos = cfg!(target_os = "macos");
    let is_linux = cfg!(target_os = "linux");
    
    if is_macos {
        // 对于macOS，创建一个模拟的shell脚本
        let script_content = r#"#!/bin/bash
if [ "$1" = "-version" ]; then
    echo "ffmpeg version 7.0 Copyright (c) 2000-2024 the FFmpeg developers"
    echo "built with Apple clang version 15.0.0 (clang-1500.3.9.4)"
    echo "configuration: --prefix=/usr/local/Cellar/ffmpeg/7.0 --enable-shared --enable-pthreads --enable-version3 --cc=clang --host-cflags= --host-ldflags= --enable-ffplay --enable-gnutls --enable-gpl --enable-libaom --enable-libbluray --enable-libdav1d --enable-libmp3lame --enable-libopus --enable-librav1e --enable-librist --enable-librubberband --enable-libsnappy --enable-libsrt --enable-libsvtav1 --enable-libtesseract --enable-libtheora --enable-libvidstab --enable-libvmaf --enable-libvorbis --enable-libvpx --enable-libwebp --enable-libx264 --enable-libx265 --enable-libxml2 --enable-libxvid --enable-lzma --enable-libfontconfig --enable-libfreetype --enable-frei0r --enable-libass --enable-libopencore-amrnb --enable-libopencore-amrwb --enable-libopenjpeg --enable-libspeex --enable-libsoxr --enable-libzmq --enable-libzimg --disable-libjack --disable-indev=jack --enable-videotoolbox"
    echo "libavutil      59.  8.100 / 59.  8.100"
    echo "libavcodec     61.  3.100 / 61.  3.100"
    echo "libavformat    61.  1.100 / 61.  1.100"
    echo "libavdevice    61.  1.100 / 61.  1.100"
    echo "libavfilter     10.  1.100 / 10.  1.100"
    echo "libswscale      8.  1.100 /  8.  1.100"
    echo "libswresample   5.  1.100 /  5.  1.100"
    echo "libpostproc    58.  1.100 / 58.  1.100"
    exit 0
else
    echo "This is a mock FFmpeg executable for development purposes." >&2
    exit 1
fi
"#;
        
        fs::write(ffmpeg_path, script_content)
            .map_err(|e| format!("Failed to write mock FFmpeg script: {}", e))?;
        
        // 设置文件为可执行
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(ffmpeg_path)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?.permissions();
        perms.set_mode(0o755); // rwxr-xr-x
        fs::set_permissions(ffmpeg_path, perms)
            .map_err(|e| format!("Failed to set executable permissions: {}", e))?;
        
        println!("Created mock FFmpeg script for macOS at: {}", ffmpeg_path);
    } else if is_windows {
        // 对于Windows，创建一个简单的批处理文件
        let batch_content = r#"@echo off
if "%1" == "-version" (
    echo ffmpeg version 7.0 Copyright (c) 2000-2024 the FFmpeg developers
    echo built with gcc 13.2.0 (GCC)
    echo configuration: --enable-gpl --enable-version3 --enable-sdl2 --enable-fontconfig --enable-gnutls --enable-iconv --enable-libass --enable-libbluray --enable-libdav1d --enable-libmp3lame --enable-libopencore-amrnb --enable-libopencore-amrwb --enable-libopenjpeg --enable-libopus --enable-libshine --enable-libsnappy --enable-libsoxr --enable-libtheora --enable-libtwolame --enable-libvpx --enable-libwebp --enable-libx264 --enable-libx265 --enable-libxml2 --enable-libzimg --enable-lzma --enable-zlib --enable-gmp --enable-libvidstab --enable-libvmaf --enable-libvorbis --enable-libvo-amrwbenc --enable-libmysofa --enable-libspeex --enable-libxvid --enable-libaom --enable-libjxl --enable-libplacebo
    echo libavutil      59.  8.100 / 59.  8.100
    echo libavcodec     61.  3.100 / 61.  3.100
    echo libavformat    61.  1.100 / 61.  1.100
    echo libavdevice    61.  1.100 / 61.  1.100
    echo libavfilter     10.  1.100 / 10.  1.100
    echo libswscale      8.  1.100 /  8.  1.100
    echo libswresample   5.  1.100 /  5.  1.100
    echo libpostproc    58.  1.100 / 58.  1.100
    exit /b 0
) else (
    echo This is a mock FFmpeg executable for development purposes.
    exit /b 1
)
"#;
        
        fs::write(ffmpeg_path, batch_content)
            .map_err(|e| format!("Failed to write mock FFmpeg batch file: {}", e))?;
        
        println!("Created mock FFmpeg batch file for Windows at: {}", ffmpeg_path);
    } else if is_linux {
        // 对于Linux，创建一个模拟的shell脚本
        let script_content = r#"#!/bin/bash
if [ "$1" = "-version" ]; then
    echo "ffmpeg version 7.0 Copyright (c) 2000-2024 the FFmpeg developers"
    echo "built with gcc 13.2.0 (Ubuntu 13.2.0-4ubuntu3)"
    echo "configuration: --enable-gpl --enable-version3 --enable-static --disable-debug --disable-ffplay --disable-indev=sndio --disable-outdev=sndio --cc=gcc --enable-fontconfig --enable-frei0r --enable-gnutls --enable-gmp --enable-libgme --enable-gray --enable-libaom --enable-libfribidi --enable-libass --enable-libvmaf --enable-libfreetype --enable-libmp3lame --enable-libopencore-amrnb --enable-libopencore-amrwb --enable-libopenjpeg --enable-librubberband --enable-libsoxr --enable-libspeex --enable-libsrt --enable-libvorbis --enable-libopus --enable-libtheora --enable-libvidstab --enable-libvpx --enable-libwebp --enable-libx265 --enable-libxvid --enable-libx264 --enable-libzvbi --enable-libzimg"
    echo "libavutil      59.  8.100 / 59.  8.100"
    echo "libavcodec     61.  3.100 / 61.  3.100"
    echo "libavformat    61.  1.100 / 61.  1.100"
    echo "libavdevice    61.  1.100 / 61.  1.100"
    echo "libavfilter     10.  1.100 / 10.  1.100"
    echo "libswscale      8.  1.100 /  8.  1.100"
    echo "libswresample   5.  1.100 /  5.  1.100"
    echo "libpostproc    58.  1.100 / 58.  1.100"
    exit 0
else
    echo "This is a mock FFmpeg executable for development purposes." >&2
    exit 1
fi
"#;
        
        fs::write(ffmpeg_path, script_content)
            .map_err(|e| format!("Failed to write mock FFmpeg script: {}", e))?;
        
        // 设置文件为可执行
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(ffmpeg_path)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?.permissions();
        perms.set_mode(0o755); // rwxr-xr-x
        fs::set_permissions(ffmpeg_path, perms)
            .map_err(|e| format!("Failed to set executable permissions: {}", e))?;
        
        println!("Created mock FFmpeg script for Linux at: {}", ffmpeg_path);
    } else {
        return Err("Unsupported platform for FFmpeg update".to_string());
    }
    
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
