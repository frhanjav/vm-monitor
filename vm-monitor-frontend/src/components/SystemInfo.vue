<template>
  <div class="system-info-row">
    <div class="system-info-card chart-card">
      <h3>System Info Snapshot (Latest)</h3>
      <div v-if="systemInfo" class="system-info-grid">
        <div class="info-item">
          <strong>Hostname:</strong>
          <span>{{ systemInfo.hostname }}</span>
        </div>
        <div class="info-item">
          <strong>OS:</strong>
          <span>{{ systemInfo.os_name }} {{ systemInfo.os_version }}</span>
        </div>
        <div class="info-item">
          <strong>Kernel:</strong>
          <span>{{ systemInfo.kernel_version }}</span>
        </div>
        <div class="info-item">
          <strong>Uptime:</strong>
          <span>{{ formatUptime(systemInfo.uptime) }}</span>
        </div>
        <div class="info-item">
          <strong>Cloud:</strong>
          <span>{{ systemInfo.cloud_provider || 'N/A' }}</span>
        </div>
        <div class="info-item">
          <strong>Total Memory:</strong>
          <span>{{ formatMemory(systemInfo.total_memory_bytes) }}</span>
        </div>
        <div class="info-item">
          <strong>Available Memory:</strong>
          <span>{{ formatMemory(systemInfo.available_memory_bytes) }}</span>
        </div>
        <div class="info-item">
          <strong>CPU Cores:</strong>
          <span>{{ systemInfo.cpu_core_count }}</span>
        </div>
      </div>
      <p v-else>No system info available.</p>
    </div>
  </div>
</template>

<script setup>
import { formatMemory, formatUptime } from '../utils';

defineProps({
  systemInfo: {
    type: Object,
    default: null,
  },
});
</script>

<style scoped>
.system-info-row {
  display: flex;
  flex-direction: column;
}

.chart-card {
  background-color: #ffffff;
  padding: 1.25rem;
  border-radius: 0.5rem;
  box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  display: flex;
  flex-direction: column;
}

.chart-card h3 {
  margin-top: 0;
  margin-bottom: 1rem;
  text-align: left;
  color: #1f2937;
  font-size: 1rem;
  font-weight: 600;
}

.system-info-grid {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  font-size: 0.875rem;
  width: 100%;
}

.info-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.375rem 0;
  border-bottom: 1px solid #f3f4f6;
  min-width: 0;
}

.info-item:last-child {
  border-bottom: none;
}

.info-item strong {
  color: #4b5563;
  margin-right: 0.75rem;
  font-weight: 500;
  white-space: nowrap;
}

.info-item span {
  color: #1f2937;
  text-align: right;
  word-break: break-all;
}

@media (min-width: 768px) {
  .system-info-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 0.5rem 1.5rem;
  }
}
</style>