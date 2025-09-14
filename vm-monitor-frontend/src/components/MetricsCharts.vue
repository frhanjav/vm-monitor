<template>
  <div class="charts-row">
    <div class="chart-card">
      <h3>CPU Usage (%)</h3>
      <Line :data="cpuChartData" :options="chartOptions('CPU Usage %')" />
    </div>
    <div class="chart-card">
      <h3>Memory Usage (Used GB)</h3>
      <Line :data="memoryChartData" :options="chartOptions('Used Memory (GB)')" />
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue';
import { Line } from 'vue-chartjs';

const props = defineProps({
  processedMetrics: {
    type: Object,
    required: true,
  },
});

const cpuChartData = computed(() => ({
  labels: props.processedMetrics.timestamps,
  datasets: [
    {
      label: 'CPU Usage %',
      backgroundColor: 'rgba(239, 68, 68, 0.3)',
      borderColor: 'rgb(239, 68, 68)',
      data: props.processedMetrics.cpuUsage,
      tension: 0.2,
      pointRadius: 2,
      borderWidth: 1.5,
    },
  ],
}));

const memoryChartData = computed(() => ({
  labels: props.processedMetrics.timestamps,
  datasets: [
    {
      label: 'Used Memory (GB)',
      backgroundColor: 'rgba(59, 130, 246, 0.3)',
      borderColor: 'rgb(59, 130, 246)',
      data: props.processedMetrics.memoryUsedGB,
      tension: 0.2,
      pointRadius: 2,
      borderWidth: 1.5,
    },
  ],
}));

const commonChartOptionsTemplate = (yAxisLabelText) => ({
  responsive: true,
  maintainAspectRatio: false,
  layout: {
    padding: {
      bottom: 20,
    },
  },
  plugins: {
    legend: {
      display: false,
    },
    title: {
      display: false,
    },
    tooltip: {
      callbacks: {
        label: function (context) {
          let label = context.dataset.label || '';
          if (label) {
            label += ': ';
          }
          if (context.parsed.y !== null) {
            label += context.parsed.y.toFixed(2);
            if (context.dataset.label && context.dataset.label.includes('%')) {
              label += '%';
            } else if (
              context.dataset.label &&
              context.dataset.label.includes('GB')
            ) {
              label += ' GB';
            }
          }
          return label;
        },
      },
    },
  },
  scales: {
    x: {
      type: 'time',
      time: {
        unit: 'minute',
        tooltipFormat: 'MMM d, HH:mm:ss',
        displayFormats: {
          minute: 'HH:mm',
          hour: 'HH:00',
          day: 'MMM d',
        },
      },
      grid: {
        display: false,
      },
      ticks: {
        maxRotation: 0,
        autoSkipPadding: 20,
        font: { size: 10 },
      },
      title: {
        display: true,
        text: 'Time',
        font: {
          size: 12,
          weight: 'normal',
        },
        padding: { top: 10, left: 0, right: 0, bottom: 0 },
      },
    },
    y: {
      beginAtZero: true,
      grace: '10%',
      grid: {
        color: '#e5e7eb',
      },
      ticks: {
        font: { size: 10 },
      },
      title: {
        display: true,
        text: yAxisLabelText,
        font: {
          size: 12,
          weight: 'normal',
        },
        padding: { top: 0, left: 0, right: 0, bottom: 10 },
      },
    },
  },
  animation: {
    duration: 0,
  },
  elements: {
    line: {
      cubicInterpolationMode: 'monotone',
    },
  },
});

const chartOptions = (yAxisLabel) => commonChartOptionsTemplate(yAxisLabel);
</script>

<style scoped>
.charts-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
  gap: 1.5rem;
}

.chart-card {
  background-color: #ffffff;
  padding: 1.25rem;
  border-radius: 0.5rem;
  box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  display: flex;
  flex-direction: column;
  height: 320px;
}

.chart-card h3 {
  margin-top: 0;
  margin-bottom: 1rem;
  text-align: left;
  color: #1f2937;
  font-size: 1rem;
  font-weight: 600;
}
</style>