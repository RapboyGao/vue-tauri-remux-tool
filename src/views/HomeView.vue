<template>
  <div class="home-container">
    <header class="app-header">
      <div class="header-content">
        <h1>蓝光转码工具</h1>
        <div class="ffmpeg-status" v-if="ffmpegInfo">
          {{ ffmpegInfo }}
        </div>
      </div>
      <div class="header-actions">
        <button @click="checkFFmpeg" class="btn btn-secondary">
          检查FFmpeg
        </button>
        <button @click="updateFFmpeg" class="btn btn-primary">
          更新FFmpeg
        </button>
      </div>
    </header>

    <main class="main-content">
      <section class="input-section">
        <h2>添加转换任务</h2>
        <div class="input-grid">
          <div class="input-group">
            <label for="source-path">蓝光文件夹</label>
            <div class="file-selector">
              <input 
                type="text" 
                id="source-path" 
                v-model="sourcePath" 
                placeholder="选择蓝光文件夹"
              />
              <button @click="selectSourceFolder" class="btn btn-primary">
                浏览
              </button>
            </div>
          </div>

          <div class="input-group">
            <label for="output-path">输出路径</label>
            <div class="file-selector">
              <input 
                type="text" 
                id="output-path" 
                v-model="outputPath" 
                placeholder="选择输出路径"
              />
              <button @click="selectOutputFolder" class="btn btn-primary">
                浏览
              </button>
            </div>
          </div>

          <div class="input-group options-group">
            <h3>转换选项</h3>
            <div class="options-grid">
              <div class="option-item">
                <input 
                  type="checkbox" 
                  id="copy-all-tracks" 
                  v-model="conversionOptions.copyAllTracks"
                />
                <label for="copy-all-tracks">复制所有音轨和字幕</label>
              </div>
              <div class="option-item">
                <input 
                  type="checkbox" 
                  id="include-chapters" 
                  v-model="conversionOptions.includeChapters"
                />
                <label for="include-chapters">保留章节信息</label>
              </div>
              <div class="option-item">
                <input 
                  type="checkbox" 
                  id="include-metadata" 
                  v-model="conversionOptions.includeMetadata"
                />
                <label for="include-metadata">保留元数据</label>
              </div>
            </div>
          </div>

          <div class="action-buttons">
            <button @click="addTask" class="btn btn-primary btn-large">
              添加到转换队列
            </button>
          </div>
        </div>
      </section>

      <section class="queue-section">
        <h2>转换队列</h2>
        <div class="queue-list">
          <div 
            v-for="task in tasks" 
            :key="task.id" 
            class="task-item"
            :class="`task-${task.status}`"
          >
            <div class="task-info">
              <div class="task-name">{{ task.bluRayInfo.name }}</div>
              <div class="task-status">{{ getStatusText(task.status) }}</div>
            </div>
            <div class="task-progress">
              <div class="progress-bar">
                <div 
                  class="progress-fill" 
                  :style="{ width: `${task.progress}%` }"
                ></div>
              </div>
              <div class="progress-text">{{ Math.round(task.progress) }}%</div>
            </div>
            <div class="task-actions">
              <button 
                v-if="task.status === TaskStatus.PENDING || task.status === TaskStatus.PAUSED"
                @click="startTask(task.id)"
                class="btn btn-success"
              >
                开始
              </button>
              <button 
                v-else-if="task.status === TaskStatus.RUNNING"
                @click="pauseTask(task.id)"
                class="btn btn-warning"
              >
                暂停
              </button>
              <button 
                v-if="task.status === TaskStatus.RUNNING || task.status === TaskStatus.PAUSED"
                @click="cancelTask(task.id)"
                class="btn btn-danger"
              >
                取消
              </button>
              <button 
                v-if="task.status === TaskStatus.COMPLETED || task.status === TaskStatus.FAILED"
                @click="removeTask(task.id)"
                class="btn btn-secondary"
              >
                删除
              </button>
            </div>
          </div>
          <div v-if="tasks.length === 0" class="empty-queue">
            队列中没有任务，请添加转换任务
          </div>
        </div>
      </section>
    </main>

    <footer class="app-footer">
      <p>© 2026 蓝光转码工具</p>
    </footer>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { ConversionTask, TaskStatus, ConversionOptions } from '../types';
import { ffmpegService } from '../services/ffmpeg';
import { bluRayService } from '../services/bluray';
import { conversionService } from '../services/conversion';

// 状态管理
const sourcePath = ref('');
const outputPath = ref('');
const ffmpegInfo = ref('');
const tasks = ref<ConversionTask[]>([]);

// 转换选项
const conversionOptions = ref<ConversionOptions>({
  copyAllTracks: true,
  includeChapters: true,
  includeMetadata: true,
  outputFormat: 'mkv'
});

// 生命周期钩子
onMounted(() => {
  loadTasks();
  checkFFmpegVersion();
});

// 加载任务列表
function loadTasks() {
  tasks.value = conversionService.getTasks();
}

// 检查FFmpeg版本
async function checkFFmpegVersion() {
  const info = await ffmpegService.getFFmpegInfo();
  ffmpegInfo.value = `FFmpeg版本: ${info.version} (${info.isInstalled ? '已安装' : '未安装'})`;
}

// 检查FFmpeg
async function checkFFmpeg() {
  await checkFFmpegVersion();
  alert(ffmpegInfo.value);
}

// 更新FFmpeg
async function updateFFmpeg() {
  const result = await ffmpegService.updateFFmpeg();
  if (result) {
    await checkFFmpegVersion();
    alert('FFmpeg更新成功！');
  } else {
    alert('FFmpeg更新失败！');
  }
}

// 选择源文件夹
async function selectSourceFolder() {
  // 使用Tauri的invoke API调用Rust的文件选择对话框
  const selected = await invoke<string>('select_directory', {
    title: '选择蓝光文件夹'
  });

  if (selected) {
    sourcePath.value = selected;
  }
}

// 选择输出文件夹
async function selectOutputFolder() {
  // 使用Tauri的invoke API调用Rust的文件选择对话框
  const selected = await invoke<string>('select_directory', {
    title: '选择输出路径'
  });

  if (selected) {
    outputPath.value = selected;
  }
}

// 添加转换任务
async function addTask() {
  if (!sourcePath.value || !outputPath.value) {
    alert('请选择蓝光文件夹和输出路径！');
    return;
  }

  // 检测蓝光文件夹结构
  const isBluRay = await bluRayService.detectStructure(sourcePath.value);
  if (!isBluRay) {
    alert('选择的文件夹不是有效的蓝光文件夹！');
    return;
  }

  // 获取蓝光文件夹信息
  const bluRayInfo = await bluRayService.getBluRayInfo(sourcePath.value);
  if (!bluRayInfo.mainPlaylist) {
    alert('无法找到蓝光文件夹中的主播放列表！');
    return;
  }

  // 创建转换任务
  conversionService.createTask(sourcePath.value, outputPath.value, bluRayInfo);
  loadTasks();
  
  // 清空输入
  sourcePath.value = '';
}

// 开始任务
async function startTask(taskId: string) {
  await conversionService.startTask(taskId, conversionOptions.value);
  loadTasks();
}

// 暂停任务
async function pauseTask(taskId: string) {
  await conversionService.pauseTask(taskId);
  loadTasks();
}

// 取消任务
async function cancelTask(taskId: string) {
  await conversionService.cancelTask(taskId);
  loadTasks();
}

// 删除任务
function removeTask(taskId: string) {
  conversionService.deleteTask(taskId);
  loadTasks();
}

// 获取状态文本
function getStatusText(status: TaskStatus): string {
  const statusMap: Record<TaskStatus, string> = {
    [TaskStatus.PENDING]: '等待中',
    [TaskStatus.RUNNING]: '转换中',
    [TaskStatus.PAUSED]: '已暂停',
    [TaskStatus.COMPLETED]: '已完成',
    [TaskStatus.FAILED]: '转换失败',
    [TaskStatus.CANCELLED]: '已取消'
  };
  return statusMap[status] || '未知状态';
}
</script>

<style scoped>
.home-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}

.app-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 30px;
  padding-bottom: 15px;
  border-bottom: 1px solid #e0e0e0;
}

.header-content {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.app-header h1 {
  margin: 0;
  font-size: 24px;
  color: #333;
}

.ffmpeg-status {
  font-size: 14px;
  color: #666;
  font-weight: 500;
}

.header-actions {
  display: flex;
  gap: 10px;
}

.main-content {
  display: flex;
  flex-direction: column;
  gap: 30px;
}

.input-section {
  background: #f8f9fa;
  padding: 20px;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.input-section h2 {
  margin-top: 0;
  margin-bottom: 20px;
  font-size: 18px;
  color: #333;
}

.input-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}

.input-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.input-group label {
  font-weight: 500;
  color: #555;
}

.file-selector {
  display: flex;
  gap: 10px;
}

.file-selector input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}

.options-group {
  grid-column: 1 / -1;
}

.options-group h3 {
  margin: 0 0 15px 0;
  font-size: 16px;
  color: #333;
}

.options-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 15px;
}

.option-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.option-item label {
  font-weight: normal;
  cursor: pointer;
}

.action-buttons {
  grid-column: 1 / -1;
  display: flex;
  justify-content: center;
  margin-top: 10px;
}

.queue-section {
  background: #f8f9fa;
  padding: 20px;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.queue-section h2 {
  margin-top: 0;
  margin-bottom: 20px;
  font-size: 18px;
  color: #333;
}

.queue-list {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.task-item {
  background: white;
  padding: 15px;
  border-radius: 8px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.task-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.task-name {
  font-weight: 500;
  color: #333;
}

.task-status {
  font-size: 14px;
  font-weight: 500;
  padding: 2px 8px;
  border-radius: 12px;
}

.task-pending .task-status {
  background: #e3f2fd;
  color: #1976d2;
}

.task-running .task-status {
  background: #e8f5e8;
  color: #388e3c;
}

.task-paused .task-status {
  background: #fff8e1;
  color: #f57c00;
}

.task-completed .task-status {
  background: #e8f5e8;
  color: #388e3c;
}

.task-failed .task-status {
  background: #ffebee;
  color: #d32f2f;
}

.task-cancelled .task-status {
  background: #f5f5f5;
  color: #616161;
}

.task-progress {
  display: flex;
  align-items: center;
  gap: 10px;
}

.progress-bar {
  flex: 1;
  height: 8px;
  background: #e0e0e0;
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: #4caf50;
  border-radius: 4px;
  transition: width 0.3s ease;
}

.progress-text {
  font-size: 14px;
  color: #666;
  min-width: 40px;
  text-align: right;
}

.task-actions {
  display: flex;
  gap: 10px;
  justify-content: flex-end;
}

.empty-queue {
  text-align: center;
  padding: 30px;
  color: #999;
  font-style: italic;
  background: white;
  border-radius: 8px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-primary {
  background: #2196f3;
  color: white;
}

.btn-primary:hover {
  background: #1976d2;
}

.btn-secondary {
  background: #6c757d;
  color: white;
}

.btn-secondary:hover {
  background: #5a6268;
}

.btn-success {
  background: #4caf50;
  color: white;
}

.btn-success:hover {
  background: #388e3c;
}

.btn-warning {
  background: #ff9800;
  color: white;
}

.btn-warning:hover {
  background: #f57c00;
}

.btn-danger {
  background: #f44336;
  color: white;
}

.btn-danger:hover {
  background: #d32f2f;
}

.btn-large {
  padding: 12px 24px;
  font-size: 16px;
}

.app-footer {
  margin-top: 30px;
  padding-top: 20px;
  border-top: 1px solid #e0e0e0;
  text-align: center;
  color: #666;
  font-size: 14px;
}
</style>
