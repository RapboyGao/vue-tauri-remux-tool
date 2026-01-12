// 转换任务状态
export enum TaskStatus {
  PENDING = 'pending',
  RUNNING = 'running',
  PAUSED = 'paused',
  COMPLETED = 'completed',
  FAILED = 'failed',
  CANCELLED = 'cancelled'
}

// 媒体轨道信息
export interface MediaTrack {
  id: number;
  type: 'video' | 'audio' | 'subtitle';
  codec: string;
  language: string;
  title: string;
  duration: number;
}

// 蓝光文件夹信息
export interface BluRayInfo {
  path: string;
  name: string;
  hasBDMV: boolean;
  mainPlaylist?: string;
  tracks: MediaTrack[];
  duration: number;
}

// 转换任务
export interface ConversionTask {
  id: string;
  sourcePath: string;
  outputPath: string;
  bluRayInfo: BluRayInfo;
  status: TaskStatus;
  progress: number;
  pid?: number; // FFmpeg进程ID
  startTime?: Date;
  endTime?: Date;
  duration?: number;
  log: string[];
  error?: string;
}

// FFmpeg信息
export interface FFmpegInfo {
  version: string;
  path: string;
  isInstalled: boolean;
  isLatest: boolean;
  lastChecked: Date;
}

// 转换选项
export interface ConversionOptions {
  copyAllTracks: boolean;
  includeChapters: boolean;
  includeMetadata: boolean;
  outputFormat: 'mkv'; // 目前只支持MKV
}
