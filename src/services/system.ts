class SystemService {
  private cpuUsage: number = 0;
  private memoryUsage: number = 0;
  private monitoring: boolean = false;
  private monitorInterval: number | null = null;

  // 开始监控系统资源
  startMonitoring(interval: number = 5000): void {
    if (this.monitoring) {
      return;
    }

    this.monitoring = true;
    this.monitorInterval = setInterval(() => {
      this.updateSystemStats();
    }, interval);
  }

  // 停止监控系统资源
  stopMonitoring(): void {
    if (!this.monitoring || !this.monitorInterval) {
      return;
    }

    clearInterval(this.monitorInterval);
    this.monitoring = false;
    this.monitorInterval = null;
  }

  // 更新系统资源统计
  private async updateSystemStats(): Promise<void> {
    try {
      // 这里需要实现获取系统资源使用情况的逻辑
      // 实际应用中可以使用tauri-plugin-os或其他系统监控库
      // 这里为了简化，我们使用模拟数据
      this.cpuUsage = Math.random() * 50 + 20; // 20-70%
      this.memoryUsage = Math.random() * 30 + 40; // 40-70%
    } catch (error) {
      console.error('Failed to update system stats:', error);
    }
  }

  // 获取CPU使用率
  getCpuUsage(): number {
    return this.cpuUsage;
  }

  // 获取内存使用率
  getMemoryUsage(): number {
    return this.memoryUsage;
  }

  // 根据系统负载获取推荐的转换优先级
  getRecommendedPriority(): 'low' | 'medium' | 'high' {
    if (this.cpuUsage > 80 || this.memoryUsage > 80) {
      return 'low';
    } else if (this.cpuUsage > 60 || this.memoryUsage > 60) {
      return 'medium';
    } else {
      return 'high';
    }
  }

  // 根据系统负载调整FFmpeg参数
  adjustFfmpegArgs(args: string[]): string[] {
    const priority = this.getRecommendedPriority();
    const adjustedArgs = [...args];

    // 根据优先级调整FFmpeg的线程数
    // 这里可以根据需要添加其他优化参数
    if (priority === 'low') {
      // 低优先级，减少线程数
      adjustedArgs.push('-threads', '1');
    } else if (priority === 'medium') {
      // 中优先级，使用默认线程数
      adjustedArgs.push('-threads', '2');
    } else {
      // 高优先级，使用更多线程
      adjustedArgs.push('-threads', '0'); // 0表示使用所有可用线程
    }

    return adjustedArgs;
  }
}

export const systemService = new SystemService();
