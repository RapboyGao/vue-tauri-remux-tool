import { invoke } from '@tauri-apps/api/core';
import { FFmpegInfo } from '../types';
import { fileExists, getAppDataDir } from '../utils';

class FFmpegService {
  private ffmpegPath: string = '';
  private ffmpegInfo: FFmpegInfo | null = null;

  constructor() {
    this.init();
  }

  private async init() {
    const appDataDir = await getAppDataDir();
    // 使用条件编译或运行时检测来确定平台
    const isWindows = false; // 实际应用中应该使用Tauri的平台API
    this.ffmpegPath = `${appDataDir}/ffmpeg/ffmpeg${isWindows ? '.exe' : ''}`;
  }

  // 检查FFmpeg是否安装
  async checkInstallation(): Promise<boolean> {
    const exists = await fileExists(this.ffmpegPath);
    if (exists) {
      // 如果文件存在，验证其完整性
      return await this.verifyIntegrity();
    }
    return false;
  }

  // 获取FFmpeg版本信息
  async getVersion(): Promise<string> {
    try {
      const version = await invoke<string>('check_ffmpeg_version', {
        ffmpegPath: this.ffmpegPath
      });
      return version;
    } catch (error) {
      console.error('Failed to get FFmpeg version:', error);
      return 'unknown';
    }
  }

  // 检查FFmpeg完整性
  async verifyIntegrity(): Promise<boolean> {
    try {
      const version = await this.getVersion();
      return version !== 'unknown' && version.length > 0;
    } catch (error) {
      console.error('FFmpeg integrity check failed:', error);
      return false;
    }
  }

  // 下载并更新FFmpeg
  async updateFFmpeg(): Promise<boolean> {
    try {
      const result = await invoke<boolean>('update_ffmpeg', {
        ffmpegPath: this.ffmpegPath
      });
      
      // 重置缓存，确保下次获取信息时会重新检查
      this.resetCache();
      
      return result;
    } catch (error) {
      console.error('FFmpeg update failed:', error);
      return false;
    }
  }

  // 获取FFmpeg完整信息
  async getFFmpegInfo(): Promise<FFmpegInfo> {
    if (this.ffmpegInfo) {
      return this.ffmpegInfo;
    }

    const isInstalled = await this.checkInstallation();
    let version = 'unknown';
    
    if (isInstalled) {
      version = await this.getVersion();
    }

    this.ffmpegInfo = {
      version,
      path: this.ffmpegPath,
      isInstalled,
      isLatest: false, // 后续实现版本比较逻辑
      lastChecked: new Date()
    };

    return this.ffmpegInfo;
  }

  // 获取FFmpeg可执行文件路径
  getFFmpegPath(): string {
    return this.ffmpegPath;
  }

  // 重置FFmpeg信息缓存
  resetCache(): void {
    this.ffmpegInfo = null;
  }
}

export const ffmpegService = new FFmpegService();
