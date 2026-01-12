import { invoke } from '@tauri-apps/api/core';

// 生成唯一ID
export const generateId = (): string => {
  return Date.now().toString(36) + Math.random().toString(36).substring(2, 9);
};

// 格式化时间
export const formatTime = (seconds: number): string => {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  return `${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
};

// 格式化文件大小
export const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

// 获取文件名（不含扩展名）
export const getFileNameWithoutExt = (path: string): string => {
  const fileName = path.split(/[/\\]/).pop() || '';
  return fileName.split('.').slice(0, -1).join('.') || fileName;
};

// 获取文件扩展名
export const getFileExt = (path: string): string => {
  return path.split('.').pop() || '';
};

// 验证蓝光文件夹
export const isBluRayFolder = (path: string): boolean => {
  // 检查是否包含BDMV目录
  return path.toLowerCase().includes('bdmv') || path.toLowerCase().endsWith('.bdmv');
};

// 解析FFmpeg输出日志
export const parseFfmpegOutput = (output: string): { progress: number; time: number } => {
  const timeMatch = output.match(/time=(\d+):(\d+):(\d+).(\d+)/);
  const durationMatch = output.match(/Duration: (\d+):(\d+):(\d+).(\d+)/);
  
  if (timeMatch && durationMatch) {
    const timeSeconds = 
      parseInt(timeMatch[1]) * 3600 + 
      parseInt(timeMatch[2]) * 60 + 
      parseInt(timeMatch[3]) + 
      parseInt(timeMatch[4]) / 100;
    
    const durationSeconds = 
      parseInt(durationMatch[1]) * 3600 + 
      parseInt(durationMatch[2]) * 60 + 
      parseInt(durationMatch[3]) + 
      parseInt(durationMatch[4]) / 100;
    
    const progress = (timeSeconds / durationSeconds) * 100;
    return { progress: Math.min(100, Math.max(0, progress)), time: timeSeconds };
  }
  
  return { progress: 0, time: 0 };
};

// 检查文件是否存在
export const fileExists = async (path: string): Promise<boolean> => {
  try {
    // 使用Tauri的invoke API调用Rust的文件存在检查
    const exists = await invoke<boolean>('file_exists', {
      path
    });
    return exists;
  } catch {
    return false;
  }
};

// 获取应用数据目录
export const getAppDataDir = async (): Promise<string> => {
  const { appDataDir } = await import('@tauri-apps/api/path');
  return await appDataDir();
};
