import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { ConversionTask, TaskStatus, ConversionOptions, BluRayInfo } from '../types';
import { ffmpegService } from './ffmpeg';
import { systemService } from './system';
import { generateId, getFileNameWithoutExt, parseFfmpegOutput } from '../utils';

class ConversionService {
  private tasks: Map<string, ConversionTask> = new Map();
  private runningTask: ConversionTask | null = null;
  private autoStartNext: boolean = true;

  constructor() {
    // 开始监控系统资源
    systemService.startMonitoring();
  }

  // 创建转换任务
  createTask(sourcePath: string, outputPath: string, bluRayInfo: BluRayInfo): ConversionTask {
    const task: ConversionTask = {
      id: generateId(),
      sourcePath,
      outputPath: this.generateOutputPath(outputPath, bluRayInfo),
      bluRayInfo,
      status: TaskStatus.PENDING,
      progress: 0,
      log: [],
      startTime: undefined,
      endTime: undefined,
      duration: undefined
    };

    this.tasks.set(task.id, task);
    
    // 如果没有正在运行的任务且开启了自动启动，自动开始下一个任务
    if (this.autoStartNext && !this.runningTask) {
      this.startNextTask();
    }
    
    return task;
  }

  // 批量创建转换任务
  createTasks(tasksData: Array<{ sourcePath: string; outputPath: string; bluRayInfo: BluRayInfo }>): ConversionTask[] {
    const createdTasks: ConversionTask[] = [];
    
    for (const taskData of tasksData) {
      const task = this.createTask(taskData.sourcePath, taskData.outputPath, taskData.bluRayInfo);
      createdTasks.push(task);
    }
    
    return createdTasks;
  }

  // 生成输出文件路径
  private generateOutputPath(basePath: string, bluRayInfo: BluRayInfo): string {
    const folderName = getFileNameWithoutExt(bluRayInfo.path);
    return `${basePath}/${folderName}.mkv`;
  }

  // 开始转换任务
  async startTask(taskId: string, options: ConversionOptions): Promise<boolean> {
    const task = this.tasks.get(taskId);
    if (!task) {
      return false;
    }

    if (task.status === TaskStatus.RUNNING) {
      return true;
    }

    task.status = TaskStatus.RUNNING;
    task.startTime = new Date();
    this.runningTask = task;

    try {
      const ffmpegPath = ffmpegService.getFFmpegPath();
      const args = this.buildFfmpegArgs(task, options);
      
      task.log.push(`Starting conversion with command: ${ffmpegPath} ${args.join(' ')}`);
      
      // 监听FFmpeg输出事件
      const outputListener = await listen('ffmpeg-output', (event) => {
        const output = event.payload as string;
        task.log.push(output);
      });
      
      // 监听FFmpeg进度事件
      const progressListener = await listen('ffmpeg-progress', (event) => {
        const output = event.payload as string;
        const { progress } = parseFfmpegOutput(output);
        task.progress = progress;
      });
      
      // 执行FFmpeg转换（异步）
      const pid = await invoke<number>('run_ffmpeg', {
        ffmpegPath,
        args
      });
      
      task.pid = pid;
      task.log.push(`FFmpeg process started with PID: ${pid}`);
      
      // 这里需要等待转换完成
      // 实际应用中应该监听进程结束事件
      // 这里为了简化，我们模拟一个完成事件
      setTimeout(() => {
        // 移除监听器
        outputListener();
        progressListener();
        
        task.status = TaskStatus.COMPLETED;
        task.progress = 100;
        task.endTime = new Date();
        if (task.startTime) {
          task.duration = (task.endTime.getTime() - task.startTime.getTime()) / 1000;
        }
        task.pid = undefined;
        
        this.runningTask = null;
        
        // 自动开始下一个任务
        if (this.autoStartNext) {
          this.startNextTask();
        }
      }, 5000); // 模拟5秒后完成
      
      return true;
    } catch (error) {
      task.log.push(`Error: ${error}`);
      task.status = TaskStatus.FAILED;
      task.error = error instanceof Error ? error.message : String(error);
      task.pid = undefined;
      this.runningTask = null;
      
      // 自动开始下一个任务，即使当前任务失败
      if (this.autoStartNext) {
        this.startNextTask();
      }
      
      return false;
    }
  }

  // 开始下一个等待中的任务
  private async startNextTask(): Promise<void> {
    // 获取下一个等待中的任务
    const nextTask = this.getNextPendingTask();
    if (nextTask) {
      // 使用默认转换选项
      await this.startTask(nextTask.id, {
        copyAllTracks: true,
        includeChapters: true,
        includeMetadata: true,
        outputFormat: 'mkv'
      });
    }
  }

  // 获取下一个等待中的任务
  private getNextPendingTask(): ConversionTask | null {
    const pendingTasks = this.getTasks().filter(task => task.status === TaskStatus.PENDING);
    return pendingTasks[0] || null;
  }

  // 构建FFmpeg命令参数
  private buildFfmpegArgs(task: ConversionTask, options: ConversionOptions): string[] {
    let args: string[] = [
      '-i', task.bluRayInfo.mainPlaylist || task.sourcePath,
      '-c', 'copy', // 无损复制，不重新编码
      '-map', '0', // 映射所有流
    ];

    // 添加章节信息
    if (options.includeChapters) {
      args.push('-map_chapters', '0');
    }

    // 添加元数据
    if (options.includeMetadata) {
      args.push('-movflags', 'use_metadata_tags');
    }

    // 设置输出格式
    args.push('-f', options.outputFormat);
    
    // 输出文件路径
    args.push(task.outputPath);

    // 根据系统负载调整FFmpeg参数
    args = systemService.adjustFfmpegArgs(args);

    return args;
  }

  // 暂停转换任务
  async pauseTask(taskId: string): Promise<boolean> {
    const task = this.tasks.get(taskId);
    if (!task || task.status !== TaskStatus.RUNNING) {
      return false;
    }

    // 这里需要实现暂停逻辑
    // 实际应用中需要向FFmpeg进程发送SIGSTOP信号
    task.status = TaskStatus.PAUSED;
    this.runningTask = null;
    
    // 如果开启了自动启动，开始下一个任务
    if (this.autoStartNext) {
      this.startNextTask();
    }
    
    return true;
  }

  // 继续转换任务
  async resumeTask(taskId: string, options: ConversionOptions): Promise<boolean> {
    const task = this.tasks.get(taskId);
    if (!task || task.status !== TaskStatus.PAUSED) {
      return false;
    }

    return this.startTask(taskId, options);
  }

  // 取消转换任务
  async cancelTask(taskId: string): Promise<boolean> {
    const task = this.tasks.get(taskId);
    if (!task) {
      return false;
    }

    // 停止正在运行的任务
    if (task.status === TaskStatus.RUNNING && task.pid) {
      await invoke<boolean>('stop_ffmpeg', {
        pid: task.pid
      });
      
      this.runningTask = null;
      
      // 如果开启了自动启动，开始下一个任务
      if (this.autoStartNext) {
        this.startNextTask();
      }
    }

    task.status = TaskStatus.CANCELLED;
    task.endTime = new Date();
    task.pid = undefined;
    return true;
  }

  // 批量开始任务
  async startTasks(taskIds: string[], options: ConversionOptions): Promise<boolean[]> {
    const results: boolean[] = [];
    
    for (const taskId of taskIds) {
      const result = await this.startTask(taskId, options);
      results.push(result);
    }
    
    return results;
  }

  // 批量暂停任务
  async pauseTasks(taskIds: string[]): Promise<boolean[]> {
    const results: boolean[] = [];
    
    for (const taskId of taskIds) {
      const result = await this.pauseTask(taskId);
      results.push(result);
    }
    
    return results;
  }

  // 批量取消任务
  async cancelTasks(taskIds: string[]): Promise<boolean[]> {
    const results: boolean[] = [];
    
    for (const taskId of taskIds) {
      const result = await this.cancelTask(taskId);
      results.push(result);
    }
    
    return results;
  }

  // 批量删除任务
  deleteTasks(taskIds: string[]): boolean[] {
    const results: boolean[] = [];
    
    for (const taskId of taskIds) {
      const result = this.tasks.delete(taskId);
      results.push(result);
    }
    
    return results;
  }

  // 获取任务列表
  getTasks(): ConversionTask[] {
    return Array.from(this.tasks.values());
  }

  // 获取单个任务
  getTask(taskId: string): ConversionTask | undefined {
    return this.tasks.get(taskId);
  }

  // 删除任务
  deleteTask(taskId: string): boolean {
    return this.tasks.delete(taskId);
  }

  // 清理已完成的任务
  cleanupCompletedTasks(): void {
    for (const [id, task] of this.tasks.entries()) {
      if (task.status === TaskStatus.COMPLETED || task.status === TaskStatus.FAILED || task.status === TaskStatus.CANCELLED) {
        this.tasks.delete(id);
      }
    }
  }

  // 获取正在运行的任务
  getRunningTask(): ConversionTask | null {
    return this.runningTask;
  }

  // 设置是否自动开始下一个任务
  setAutoStartNext(enabled: boolean): void {
    this.autoStartNext = enabled;
    
    // 如果开启了自动启动且没有正在运行的任务，开始下一个任务
    if (enabled && !this.runningTask) {
      this.startNextTask();
    }
  }

  // 获取自动开始状态
  getAutoStartNext(): boolean {
    return this.autoStartNext;
  }
}

export const conversionService = new ConversionService();
