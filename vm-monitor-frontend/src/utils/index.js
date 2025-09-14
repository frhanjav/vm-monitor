// Utility functions for formatting and API calls

const MOCK_API_BASE_URL = 'http://localhost:8000';

// Formatting functions
export function formatMemory(bytes) {
  if (typeof bytes !== 'number' || isNaN(bytes)) return 'N/A';
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

export function formatUptime(seconds) {
  if (typeof seconds !== 'number' || isNaN(seconds)) return 'N/A';
  const d = Math.floor(seconds / (3600 * 24));
  const h = Math.floor((seconds % (3600 * 24)) / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  return `${d}d ${h}h ${m}m ${s}s`;
}

// API functions
export async function checkApiStatus() {
  try {
    const response = await fetch(`${MOCK_API_BASE_URL}/v1/health`);
    return {
      isConnected: response.ok,
      message: response.ok ? 'Connected' : `Error (Status: ${response.status})`,
    };
  } catch (error) {
    console.error('API health check failed:', error);
    return {
      isConnected: false,
      message: 'Disconnected (Fetch Error)',
    };
  }
}

export async function fetchAvailableAgents() {
  try {
    const response = await fetch(`${MOCK_API_BASE_URL}/admin/agents`);
    if (!response.ok) throw new Error(`Failed to fetch agents: ${response.status}`);
    const data = await response.json();
    return Object.values(data).map((agent) => ({
      ...agent,
      instance_id:
        typeof agent.instance_id === 'object'
          ? agent.instance_id.toString()
          : agent.instance_id,
    }));
  } catch (error) {
    console.error('Error fetching agents:', error);
    throw error;
  }
}

export async function fetchMetricsForAgent(instanceId) {
  if (!instanceId) {
    throw new Error('Please select an agent first.');
  }
  
  try {
    const response = await fetch(
      `${MOCK_API_BASE_URL}/admin/metrics/${instanceId}`
    );
    if (!response.ok) {
      const errData = await response
        .json()
        .catch(() => ({ detail: `HTTP error! status: ${response.status}` }));
      throw new Error(
        errData.detail || `HTTP error! status: ${response.status}`
      );
    }
    return await response.json();
  } catch (error) {
    console.error('Failed to fetch metrics:', error);
    throw error;
  }
}

// Metrics processing function
export function processMetricsBatches(metricBatches, agentCloudProvider = 'N/A') {
  const processedMetrics = {
    timestamps: [],
    cpuUsage: [],
    memoryUsedGB: [],
  };
  
  let latestSystemInfo = null;
  let allIndividualMetrics = [];

  metricBatches.forEach((batch) => {
    if (batch && Array.isArray(batch.metrics)) {
      allIndividualMetrics.push(...batch.metrics);
    } else {
      console.warn('Encountered a batch with unexpected structure:', batch);
    }
  });

  if (allIndividualMetrics.length === 0) {
    console.warn('No individual metrics found after processing batches.');
    return { processedMetrics, latestSystemInfo };
  }

  allIndividualMetrics.sort(
    (a, b) => new Date(a.timestamp) - new Date(b.timestamp)
  );

  allIndividualMetrics.forEach((metric, index) => {
    const currentTimestamp = new Date(metric.timestamp);
    processedMetrics.timestamps.push(currentTimestamp);
    processedMetrics.cpuUsage.push(metric.cpu_metrics.usage_percent);
    processedMetrics.memoryUsedGB.push(
      parseFloat(
        (metric.memory_metrics.used_memory / (1024 * 1024 * 1024)).toFixed(2)
      )
    );

    if (index === allIndividualMetrics.length - 1) {
      latestSystemInfo = {
        hostname: metric.system_info.hostname,
        os_name: metric.system_info.os_name,
        os_version: metric.system_info.os_version,
        kernel_version: metric.system_info.kernel_version,
        uptime: metric.system_info.uptime,
        cloud_provider: agentCloudProvider,
        total_memory_bytes: metric.memory_metrics.total_memory,
        available_memory_bytes: metric.memory_metrics.available_memory,
        cpu_core_count: metric.cpu_metrics.core_count,
      };
    }
  });

  return { processedMetrics, latestSystemInfo };
}