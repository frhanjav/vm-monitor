import { ref, reactive, computed, onMounted } from 'vue';
import { 
  checkApiStatus,
  fetchAvailableAgents,
  fetchMetricsForAgent,
  processMetricsBatches
} from '../utils';

export function useMetrics() {
  // API status
  const isApiConnected = ref(false);
  const apiStatusMessage = ref('Checking...');

  // Agents
  const availableAgents = ref([]);
  const selectedInstanceId = ref('');
  const isLoadingAgents = ref(false);

  // Metrics
  const isLoadingMetrics = ref(false);
  const metricsError = ref('');
  const rawMetricsBatches = ref([]);
  const processedMetrics = reactive({
    timestamps: [],
    cpuUsage: [],
    memoryUsedGB: [],
  });
  const latestSystemInfo = ref(null);
  const agentCloudProvider = ref('');
  const lastRefreshedTime = ref(null);

  // Computed
  const selectedAgentName = computed(() => {
    const agent = availableAgents.value.find(
      (a) => a.instance_id === selectedInstanceId.value
    );
    return agent ? agent.instance_name : 'Unknown Agent';
  });

  // API Status
  async function updateApiStatus() {
    const status = await checkApiStatus();
    isApiConnected.value = status.isConnected;
    apiStatusMessage.value = status.message;
  }

  // Agents
  async function refreshAvailableAgents() {
    isLoadingAgents.value = true;
    try {
      const agents = await fetchAvailableAgents();
      availableAgents.value = agents;
    } catch (error) {
      console.error('Error fetching agents:', error);
      availableAgents.value = [];
    } finally {
      isLoadingAgents.value = false;
    }
  }

  function handleAgentSelectionChange() {
    if (selectedInstanceId.value) {
      refreshMetricsForSelectedAgent();
    } else {
      resetProcessedMetrics();
      rawMetricsBatches.value = [];
    }
  }

  // Metrics
  function resetProcessedMetrics() {
    processedMetrics.timestamps = [];
    processedMetrics.cpuUsage = [];
    processedMetrics.memoryUsedGB = [];
    latestSystemInfo.value = null;
  }

  async function refreshMetricsForSelectedAgent() {
    if (!selectedInstanceId.value) {
      metricsError.value = 'Please select an agent first.';
      return;
    }
    
    isLoadingMetrics.value = true;
    metricsError.value = '';
    
    try {
      const data = await fetchMetricsForAgent(selectedInstanceId.value);
      rawMetricsBatches.value = data;
      
      const agentInfo = availableAgents.value.find(
        (a) => a.instance_id === selectedInstanceId.value
      );
      agentCloudProvider.value = agentInfo ? agentInfo.cloud_provider : 'N/A';
      
      const { processedMetrics: processed, latestSystemInfo: systemInfo } = 
        processMetricsBatches(data, agentCloudProvider.value);
      
      // Update reactive object
      processedMetrics.timestamps = processed.timestamps;
      processedMetrics.cpuUsage = processed.cpuUsage;
      processedMetrics.memoryUsedGB = processed.memoryUsedGB;
      latestSystemInfo.value = systemInfo;
      
      lastRefreshedTime.value = new Date();
    } catch (error) {
      console.error('Failed to fetch metrics:', error);
      metricsError.value = error.message;
    } finally {
      isLoadingMetrics.value = false;
    }
  }

  // Initialize on mount
  onMounted(() => {
    updateApiStatus();
    refreshAvailableAgents();
  });

  return {
    // API status
    isApiConnected,
    apiStatusMessage,
    updateApiStatus,
    
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
    resetProcessedMetrics,
  };
}