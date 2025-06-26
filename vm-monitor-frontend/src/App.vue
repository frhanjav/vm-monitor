<template>
  <div id="app-container">
    <header>
      <h1>VM Monitor Dashboard</h1>
      <p
        class="api-status"
        :class="{ connected: isApiConnected, disconnected: !isApiConnected }"
      >
        API Status: {{ apiStatusMessage }}
      </p>
    </header>

    <main>
      <section class="agent-selection">
        <h2>Select Agent</h2>
        <div class="controls">
          <select
            v-model="selectedInstanceId"
            @change="handleAgentSelectionChange"
            :disabled="isLoadingAgents || availableAgents.length === 0"
          >
            <option disabled value="">-- Please select an agent --</option>
            <option
              v-for="agent in availableAgents"
              :key="agent.instance_id"
              :value="agent.instance_id"
            >
              {{ agent.instance_name }} ({{
                agent.instance_id.substring(0, 8)
              }}...)
            </option>
          </select>
          <button
            @click="fetchAvailableAgents"
            :disabled="isLoadingAgents"
            class="button-secondary"
          >
            {{
              isLoadingAgents ? "Refreshing Agents..." : "Refresh Agent List"
            }}
          </button>
          <button
            @click="fetchMetricsForSelectedAgent"
            :disabled="!selectedInstanceId || isLoadingMetrics"
            v-if="selectedInstanceId"
            class="button-primary"
          >
            {{
              isLoadingMetrics ? "Loading Metrics..." : "Refresh Metrics Data"
            }}
          </button>
        </div>
        <p v-if="isLoadingAgents">Loading available agents...</p>
        <p v-if="!isLoadingAgents && availableAgents.length === 0">
          No agents found. Ensure agents are registered with the API.
        </p>
      </section>

      <section
        v-if="selectedInstanceId && !isLoadingMetrics"
        class="metrics-display"
      >
        <h2>Metrics for: {{ selectedAgentName }}</h2>
        <p v-if="lastRefreshedTime">
          Last refreshed: {{ lastRefreshedTime.toLocaleTimeString() }}
        </p>
        <div v-if="metricsError" class="error-message">
          <p>Error loading metrics: {{ metricsError }}</p>
        </div>
        <!-- MODIFIED charts-grid structure -->
        <div
          v-else-if="processedMetrics.timestamps.length > 0"
          class="charts-and-info-container"
        >
          <div class="charts-row">
            <div class="chart-card">
              <h3>CPU Usage (%)</h3>
              <Line
                :data="cpuChartData"
                :options="chartOptions('CPU Usage %')"
              />
            </div>
            <div class="chart-card">
              <h3>Memory Usage (Used GB)</h3>
              <Line
                :data="memoryChartData"
                :options="chartOptions('Used Memory (GB)')"
              />
            </div>
          </div>
          <div class="system-info-row">
            <div class="system-info-card chart-card">
              <!-- Re-using chart-card style for consistency -->
              <h3>System Info Snapshot (Latest)</h3>
              <div v-if="latestSystemInfo" class="system-info-grid">
                <!-- Changed ul to div and added class -->
                <div class="info-item">
                  <strong>Hostname:</strong>
                  <span>{{ latestSystemInfo.hostname }}</span>
                </div>
                <div class="info-item">
                  <strong>OS:</strong>
                  <span
                    >{{ latestSystemInfo.os_name }}
                    {{ latestSystemInfo.os_version }}</span
                  >
                </div>
                <div class="info-item">
                  <strong>Kernel:</strong>
                  <span>{{ latestSystemInfo.kernel_version }}</span>
                </div>
                <div class="info-item">
                  <strong>Uptime:</strong>
                  <span>{{ formatUptime(latestSystemInfo.uptime) }}</span>
                </div>
                <div class="info-item">
                  <strong>Cloud:</strong>
                  <span>{{ latestSystemInfo.cloud_provider || "N/A" }}</span>
                </div>
                <!-- Add more items here, they will flow into two columns -->
                <div class="info-item">
                  <strong>Total Memory:</strong>
                  <span>{{
                    formatMemory(latestSystemInfo.total_memory_bytes)
                  }}</span>
                </div>
                <div class="info-item">
                  <strong>Available Memory:</strong>
                  <span>{{
                    formatMemory(latestSystemInfo.available_memory_bytes)
                  }}</span>
                </div>
                <div class="info-item">
                  <strong>CPU Cores:</strong>
                  <span>{{ latestSystemInfo.cpu_core_count }}</span>
                </div>
              </div>
              <p v-else>No system info available.</p>
            </div>
          </div>
        </div>
        <p v-else-if="selectedInstanceId && !isLoadingMetrics && !metricsError">
          No metrics data received yet for this agent, or data is not in the
          expected format. Ensure the vm-monitor agent is running and sending
          data.
        </p>
      </section>
      <p v-if="isLoadingMetrics && selectedInstanceId">
        Loading metrics data for {{ selectedAgentName }}...
      </p>
    </main>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted, computed } from "vue";
import { Line } from "vue-chartjs";
import {
  Chart as ChartJS,
  Title,
  Tooltip,
  Legend,
  LineElement,
  CategoryScale,
  LinearScale,
  PointElement,
  TimeScale,
} from "chart.js";
import "chartjs-adapter-date-fns";

ChartJS.register(
  Title,
  Tooltip,
  Legend,
  LineElement,
  CategoryScale,
  LinearScale,
  PointElement,
  TimeScale
);

const MOCK_API_BASE_URL = "http://localhost:8000";

const isApiConnected = ref(false);
const apiStatusMessage = ref("Checking...");

const availableAgents = ref([]);
const selectedInstanceId = ref("");
const isLoadingAgents = ref(false);

const isLoadingMetrics = ref(false);
const metricsError = ref("");
const rawMetricsBatches = ref([]);
const processedMetrics = reactive({
  timestamps: [],
  cpuUsage: [],
  memoryUsedGB: [],
});
const latestSystemInfo = ref(null);
const agentCloudProvider = ref("");
const lastRefreshedTime = ref(null);

const selectedAgentName = computed(() => {
  const agent = availableAgents.value.find(
    (a) => a.instance_id === selectedInstanceId.value
  );
  return agent ? agent.instance_name : "Unknown Agent";
});

async function checkApiStatus() {
  try {
    const response = await fetch(`${MOCK_API_BASE_URL}/v1/health`);
    isApiConnected.value = response.ok;
    apiStatusMessage.value = response.ok
      ? "Connected"
      : `Error (Status: ${response.status})`;
  } catch (error) {
    console.error("API health check failed:", error);
    isApiConnected.value = false;
    apiStatusMessage.value = "Disconnected (Fetch Error)";
  }
}

async function fetchAvailableAgents() {
  isLoadingAgents.value = true;
  try {
    const response = await fetch(`${MOCK_API_BASE_URL}/admin/agents`);
    if (!response.ok)
      throw new Error(`Failed to fetch agents: ${response.status}`);
    const data = await response.json();
    availableAgents.value = Object.values(data).map((agent) => ({
      ...agent,
      instance_id:
        typeof agent.instance_id === "object"
          ? agent.instance_id.toString()
          : agent.instance_id,
    }));
  } catch (error) {
    console.error("Error fetching agents:", error);
    availableAgents.value = [];
  } finally {
    isLoadingAgents.value = false;
  }
}

async function handleAgentSelectionChange() {
  if (selectedInstanceId.value) {
    await fetchMetricsForSelectedAgent();
  } else {
    resetProcessedMetrics();
    rawMetricsBatches.value = [];
  }
}

async function fetchMetricsForSelectedAgent() {
  if (!selectedInstanceId.value) {
    metricsError.value = "Please select an agent first.";
    return;
  }
  isLoadingMetrics.value = true;
  metricsError.value = "";
  try {
    const response = await fetch(
      `${MOCK_API_BASE_URL}/admin/metrics/${selectedInstanceId.value}`
    );
    if (!response.ok) {
      const errData = await response
        .json()
        .catch(() => ({ detail: `HTTP error! status: ${response.status}` }));
      throw new Error(
        errData.detail || `HTTP error! status: ${response.status}`
      );
    }
    const data = await response.json();
    rawMetricsBatches.value = data;
    processAllMetrics(data);
    lastRefreshedTime.value = new Date();

    const agentInfo = availableAgents.value.find(
      (a) => a.instance_id === selectedInstanceId.value
    );
    agentCloudProvider.value = agentInfo ? agentInfo.cloud_provider : "N/A";
  } catch (error) {
    console.error("Failed to fetch metrics:", error);
    metricsError.value = error.message;
  } finally {
    isLoadingMetrics.value = false;
  }
}

function resetProcessedMetrics() {
  processedMetrics.timestamps = [];
  processedMetrics.cpuUsage = [];
  processedMetrics.memoryUsedGB = [];
  // REMOVED networkData reset
  latestSystemInfo.value = null;
}

function processAllMetrics(metricBatches) {
  resetProcessedMetrics();
  let allIndividualMetrics = [];

  metricBatches.forEach((batch) => {
    if (batch && Array.isArray(batch.metrics)) {
      allIndividualMetrics.push(...batch.metrics);
    } else {
      console.warn("Encountered a batch with unexpected structure:", batch);
    }
  });

  if (allIndividualMetrics.length === 0) {
    console.warn("No individual metrics found after processing batches.");
    return;
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

    // REMOVED network processing logic

    if (index === allIndividualMetrics.length - 1) {
      latestSystemInfo.value = {
        hostname: metric.system_info.hostname,
        os_name: metric.system_info.os_name,
        os_version: metric.system_info.os_version,
        kernel_version: metric.system_info.kernel_version,
        uptime: metric.system_info.uptime,
        cloud_provider: agentCloudProvider.value,
        total_memory_bytes: metric.memory_metrics.total_memory,
        available_memory_bytes: metric.memory_metrics.available_memory,
        cpu_core_count: metric.cpu_metrics.core_count,
      };
    }
  });
}

function formatMemory(bytes) {
  if (typeof bytes !== "number" || isNaN(bytes)) return "N/A";
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["Bytes", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

function formatUptime(seconds) {
  if (typeof seconds !== "number" || isNaN(seconds)) return "N/A";
  const d = Math.floor(seconds / (3600 * 24));
  const h = Math.floor((seconds % (3600 * 24)) / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  return `${d}d ${h}h ${m}m ${s}s`;
}

const cpuChartData = computed(() => ({
  labels: processedMetrics.timestamps,
  datasets: [
    {
      label: "CPU Usage %",
      backgroundColor: "rgba(239, 68, 68, 0.3)",
      borderColor: "rgb(239, 68, 68)", // Tailwind red-500
      data: processedMetrics.cpuUsage,
      tension: 0.2,
      pointRadius: 2,
      borderWidth: 1.5,
    },
  ],
}));
const memoryChartData = computed(() => ({
  labels: processedMetrics.timestamps,
  datasets: [
    {
      label: "Used Memory (GB)",
      backgroundColor: "rgba(59, 130, 246, 0.3)",
      borderColor: "rgb(59, 130, 246)", // Tailwind blue-500
      data: processedMetrics.memoryUsedGB,
      tension: 0.2,
      pointRadius: 2,
      borderWidth: 1.5,
    },
  ],
}));

const commonChartOptionsTemplate = (yAxisLabelText) => ({
  // Renamed param for clarity
  responsive: true,
  maintainAspectRatio: false,
  layout: {
    padding: {
      bottom: 20,
    },
  },
  plugins: {
    legend: {
      display: false, // Keep legend off for cleaner individual charts
    },
    title: {
      display: false, // Main chart title is still in the <h3>
    },
    tooltip: {
      // Optional: customize tooltips
      callbacks: {
        label: function (context) {
          let label = context.dataset.label || "";
          if (label) {
            label += ": ";
          }
          if (context.parsed.y !== null) {
            label += context.parsed.y.toFixed(2); // Format y-value in tooltip
            if (context.dataset.label && context.dataset.label.includes("%")) {
              label += "%";
            } else if (
              context.dataset.label &&
              context.dataset.label.includes("GB")
            ) {
              label += " GB";
            }
          }
          return label;
        },
      },
    },
  },
  scales: {
    x: {
      type: "time",
      time: {
        unit: "minute",
        tooltipFormat: "MMM d, HH:mm:ss", // Tooltip format for time
        displayFormats: {
          // How time is displayed on the axis
          minute: "HH:mm",
          hour: "HH:00",
          day: "MMM d",
        },
      },
      grid: {
        display: false, // Keep X-axis grid lines off for cleaner look
      },
      ticks: {
        maxRotation: 0,
        autoSkipPadding: 20, // More padding to prevent label overlap
        font: { size: 10 },
      },
      title: {
        // X-axis Title
        display: true,
        text: "Time", // Label for the X-axis
        font: {
          size: 12,
          weight: "normal", // Changed from 'bold'
        },
        padding: { top: 10, left: 0, right: 0, bottom: 0 },
      },
    },
    y: {
      beginAtZero: true,
      grace: "10%", // Add 10% padding to the top of the y-axis
      grid: {
        color: "#e5e7eb", // Lighter grid lines (Tailwind gray-200)
      },
      ticks: {
        font: { size: 10 },
      },
      title: {
        // Y-axis Title
        display: true,
        text: yAxisLabelText, // Use the parameter for the Y-axis label
        font: {
          size: 12,
          weight: "normal",
        },
        padding: { top: 0, left: 0, right: 0, bottom: 10 },
      },
    },
  },
  animation: {
    duration: 0, // Disable animation for quicker updates on refresh
  },
  elements: {
    line: {
      cubicInterpolationMode: "monotone", // Smoother lines
    },
  },
});

const chartOptions = (yAxisLabel) => commonChartOptionsTemplate(yAxisLabel);

onMounted(() => {
  checkApiStatus();
  fetchAvailableAgents();
});
</script>

<style>
/* Basic Reset & Font */
body {
  margin: 0;
  font-family:
    "Inter",
    -apple-system,
    BlinkMacSystemFont,
    "Segoe UI",
    Roboto,
    Helvetica,
    Arial,
    sans-serif;
  background-color: #f3f4f6; /* Tailwind gray-100 */
  color: #1f2937; /* Tailwind gray-800 */
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

#app-container {
  max-width: 1100px; /* Slightly narrower for better focus */
  margin: 2rem auto;
  padding: 1.5rem;
}

/* Header */
header {
  border-bottom: 1px solid #d1d5db; /* Tailwind gray-300 */
  padding-bottom: 1rem;
  margin-bottom: 2rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
}
header h1 {
  margin: 0;
  color: #111827; /* Tailwind gray-900 */
  font-size: 1.875rem; /* text-3xl */
  font-weight: 700;
}
.api-status {
  padding: 0.375rem 0.75rem;
  border-radius: 0.375rem; /* rounded-md */
  font-weight: 500;
  font-size: 0.875rem; /* text-sm */
}
.api-status.connected {
  background-color: #dcfce7; /* Tailwind green-100 */
  color: #166534; /* Tailwind green-800 */
}
.api-status.disconnected {
  background-color: #fee2e2; /* Tailwind red-100 */
  color: #991b1b; /* Tailwind red-800 */
}

/* Main Content Sections */
main section {
  background-color: #ffffff; /* White card background */
  padding: 1.5rem;
  border-radius: 0.5rem; /* rounded-lg */
  box-shadow:
    0 1px 3px 0 rgba(0, 0, 0, 0.1),
    0 1px 2px -1px rgba(0, 0, 0, 0.1); /* shadow-md */
  margin-bottom: 2rem;
}
main section h2 {
  font-size: 1.25rem; /* text-xl */
  font-weight: 600;
  color: #111827;
  margin-top: 0;
  margin-bottom: 1rem;
}

/* Agent Selection Controls */
.agent-selection .controls {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  align-items: center;
  margin-bottom: 0.5rem; /* Space before loading/error messages */
}
.agent-selection select {
  padding: 0.625rem 0.875rem;
  border: 1px solid #d1d5db;
  border-radius: 0.375rem;
  font-size: 0.875rem;
  flex-grow: 1;
  min-width: 250px;
  background-color: #fff; /* Ensure select background is white */
}
.agent-selection button {
  padding: 0.625rem 1.125rem;
  color: white;
  border: none;
  border-radius: 0.375rem;
  cursor: pointer;
  font-size: 0.875rem;
  font-weight: 500;
  transition: background-color 0.15s ease-in-out;
}
.button-primary {
  background-color: #3b82f6; /* blue-500 */
}
.button-primary:hover {
  background-color: #2563eb; /* blue-600 */
}
.button-secondary {
  background-color: #6b7280; /* gray-500 */
}
.button-secondary:hover {
  background-color: #4b5563; /* gray-600 */
}
.agent-selection button:disabled {
  background-color: #e5e7eb; /* gray-200 */
  color: #9ca3af; /* gray-400 */
  cursor: not-allowed;
}
.agent-selection p {
  /* For loading/error messages */
  font-size: 0.875rem;
  color: #4b5563;
}

/* Metrics Display & Charts Layout */
.metrics-display p {
  /* For "Last refreshed" */
  font-size: 0.875rem;
  color: #6b7280;
  margin-top: -0.5rem; /* Pull it closer to H2 */
  margin-bottom: 1.25rem;
}

.charts-and-info-container {
  display: flex;
  flex-direction: column;
  gap: 1.5rem; /* Gap between chart row and info row */
}

.charts-row {
  display: grid;
  grid-template-columns: repeat(
    auto-fit,
    minmax(320px, 1fr)
  ); /* Responsive 1 or 2 columns */
  gap: 1.5rem;
}

.chart-card {
  /* Base style for all cards (charts and info) */
  background-color: #ffffff;
  padding: 1.25rem;
  border-radius: 0.5rem;
  box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05); /* shadow-sm */
  display: flex;
  flex-direction: column;
}
.chart-card h3 {
  margin-top: 0;
  margin-bottom: 1rem;
  text-align: left; /* Align title to left */
  color: #1f2937;
  font-size: 1rem; /* text-base */
  font-weight: 600;
}

/* Specific to chart cards to control their height */
.charts-row .chart-card {
  height: 320px;
}

/* System Info Card */
.system-info-card ul {
  list-style-type: none;
  padding: 0;
  margin: 0;
  font-size: 0.875rem;
  flex-grow: 1;
}
.system-info-card li {
  padding: 0.5rem 0;
  border-bottom: 1px solid #f3f4f6;
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.system-info-card li:last-child {
  border-bottom: none;
}
.system-info-card strong {
  color: #374151;
  margin-right: 0.5rem;
  font-weight: 500;
}
.system-info-card span {
  /* For the value part */
  color: #1f2937;
  text-align: right;
}

.system-info-grid {
  /* Default to a single column layout */
  display: flex; /* Using flex for single column to easily manage items */
  flex-direction: column;
  gap: 0.5rem; /* Row gap for single column */
  font-size: 0.875rem; /* text-sm */
  width: 100%;
}

.info-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.375rem 0;
  border-bottom: 1px solid #f3f4f6; /* Subtle separator */
  min-width: 0;
}

.info-item:last-child {
  /* Remove border from the very last item in single column */
  border-bottom: none;
}

.info-item strong {
  color: #4b5563; /* Tailwind gray-600 */
  margin-right: 0.75rem; /* Increased space for better readability */
  font-weight: 500;
  white-space: nowrap;
}

.info-item span {
  /* For the value part */
  color: #1f2937; /* Tailwind gray-800 */
  text-align: right;
  word-break: break-all;
}

.error-message {
  color: #991b1b;
  background-color: #fee2e2;
  padding: 1rem;
  border-radius: 0.375rem;
  border: 1px solid #fca5a5; /* red-300 */
  font-size: 0.875rem;
}

/* Apply two-column grid layout for screens wider than a certain breakpoint (e.g., 640px for sm in Tailwind) */
@media (min-width: 768px) {
  /* You can adjust this breakpoint */
  .system-info-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr); /* Two equal columns */
    gap: 0.5rem 1.5rem; /* Row gap and increased Column gap for wider screens */
  }
}
</style>
