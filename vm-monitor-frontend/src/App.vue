<template>
  <div id="app-container">
    <Header 
      :is-api-connected="isApiConnected" 
      :api-status-message="apiStatusMessage" 
    />

    <main>
      <AgentSelector
        :available-agents="availableAgents"
        v-model:selected-instance-id="selectedInstanceId"
        :is-loading-agents="isLoadingAgents"
        :is-loading-metrics="isLoadingMetrics"
        @refresh-agents="refreshAvailableAgents"
        @refresh-metrics="refreshMetricsForSelectedAgent"
        @update:selected-instance-id="handleAgentSelectionChange"
      />

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
        <div
          v-else-if="processedMetrics.timestamps.length > 0"
          class="charts-and-info-container"
        >
          <MetricsCharts :processed-metrics="processedMetrics" />
          <SystemInfo :system-info="latestSystemInfo" />
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
} from 'chart.js';
import 'chartjs-adapter-date-fns';

// Import components
import Header from './components/Header.vue';
import AgentSelector from './components/AgentSelector.vue';
import MetricsCharts from './components/MetricsCharts.vue';
import SystemInfo from './components/SystemInfo.vue';

// Import composable
import { useMetrics } from './composables/useMetrics';

// Import global styles
import './styles/global.css';

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

// Use the metrics composable
const {
  // API status
  isApiConnected,
  apiStatusMessage,
  
  // Agents
  availableAgents,
  selectedInstanceId,
  isLoadingAgents,
  selectedAgentName,
  refreshAvailableAgents,
  handleAgentSelectionChange,
  
  // Metrics
  isLoadingMetrics,
  metricsError,
  processedMetrics,
  latestSystemInfo,
  lastRefreshedTime,
  refreshMetricsForSelectedAgent,
} = useMetrics();
</script>

<!-- Styles are now moved to individual components and global.css -->
