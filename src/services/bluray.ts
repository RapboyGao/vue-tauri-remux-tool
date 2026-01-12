import { invoke } from '@tauri-apps/api/core';
import { BluRayInfo, MediaTrack } from '../types';
import { ffmpegService } from './ffmpeg';

interface DirEntry {
  name?: string;
  path: string;
  isDirectory: boolean;
}

class BluRayService {
  // 检测蓝光文件夹结构
  async detectStructure(path: string): Promise<boolean> {
    try {
      const result = await invoke<boolean>('detect_bdmv_structure', {
        path
      });
      return result;
    } catch (error) {
      console.error('Failed to detect BDMV structure:', error);
      return false;
    }
  }

  // 查找蓝光文件夹中的主播放列表
  async findMainPlaylist(path: string): Promise<string | undefined> {
    try {
      const bdmvPath = `${path}/BDMV`;
      const playlistPath = `${bdmvPath}/PLAYLIST`;
      
      // 读取PLAYLIST目录下的所有文件
      const entries = await invoke<DirEntry[]>('read_dir', {
        path: playlistPath
      });
      
      // 过滤出mpls文件
      const mplsFiles = entries
        .filter((entry: DirEntry) => entry.name?.endsWith('.mpls'))
        .map((entry: DirEntry) => entry.name || '')
        .sort();
      
      if (mplsFiles.length === 0) {
        return undefined;
      }
      
      // 通常主播放列表是最大的文件或特定命名的文件
      // 这里简单返回第一个文件，实际应用中可能需要更复杂的逻辑
      return `${playlistPath}/${mplsFiles[0]}`;
    } catch (error) {
      console.error('Failed to find main playlist:', error);
      return undefined;
    }
  }

  // 解析媒体信息
  async parseMediaInfo(inputPath: string): Promise<MediaTrack[]> {
    try {
      const ffmpegPath = ffmpegService.getFFmpegPath();
      const output = await invoke<string>('get_media_info', {
        ffmpegPath,
        inputPath
      });
      
      return this.parseFFmpegOutput(output);
    } catch (error) {
      console.error('Failed to parse media info:', error);
      return [];
    }
  }

  // 解析FFmpeg输出，提取媒体轨道信息
  private parseFFmpegOutput(output: string): MediaTrack[] {
    const tracks: MediaTrack[] = [];
    let trackId = 0;
    
    // 分割输出为行
    const lines = output.split('\n');
    
    lines.forEach(line => {
      if (line.includes('Stream #')) {
        trackId++;
        
        // 解析轨道类型
        let type: 'video' | 'audio' | 'subtitle' = 'video';
        if (line.includes('Audio:')) {
          type = 'audio';
        } else if (line.includes('Subtitle:')) {
          type = 'subtitle';
        }
        
        // 解析编解码器
        const codecMatch = line.match(/(\w+)(,|\s)/);
        const codec = codecMatch ? codecMatch[1] : 'unknown';
        
        // 解析语言
        const langMatch = line.match(/\[eng\]|\[und\]|\[(\w{3})\]/);
        const language = langMatch ? (langMatch[1] || 'eng') : 'und';
        
        // 创建轨道对象
        const track: MediaTrack = {
          id: trackId,
          type,
          codec,
          language,
          title: `Track ${trackId}`,
          duration: 0 // 后续从其他行解析
        };
        
        tracks.push(track);
      }
    });
    
    return tracks;
  }

  // 获取蓝光文件夹信息
  async getBluRayInfo(path: string): Promise<BluRayInfo> {
    const hasBDMV = await this.detectStructure(path);
    let mainPlaylist: string | undefined = undefined;
    let tracks: MediaTrack[] = [];
    let duration = 0;
    
    if (hasBDMV) {
      mainPlaylist = await this.findMainPlaylist(path);
      
      if (mainPlaylist) {
        tracks = await this.parseMediaInfo(mainPlaylist);
        // 这里应该解析实际时长，暂时设置为0
        duration = 0;
      }
    }
    
    return {
      path,
      name: path.split(/[/\\]/).pop() || '',
      hasBDMV,
      mainPlaylist,
      tracks,
      duration
    };
  }

  // 验证蓝光文件夹的完整性
  async verifyBluRayFolder(path: string): Promise<boolean> {
    const info = await this.getBluRayInfo(path);
    return info.hasBDMV && info.mainPlaylist !== null;
  }
}

export const bluRayService = new BluRayService();
