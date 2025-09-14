<template>
  <section class="agent-selection">
    <h2>Select Agent</h2>
    <div class="controls">
      <select
        :value="selectedInstanceId"
        @change="handleSelectionChange"
        :disabled="isLoadingAgents || availableAgents.length === 0"
      >
        <option disabled value="">-- Please select an agent --</option>
        <option
          v-for="agent in availableAgents"
          :key="agent.instance_id"
          :value="agent.instance_id"
        >
          {{ agent.instance_name }} ({{ agent.instance_id.substring(0, 8) }}...)
        </option>
      </select>
      <button
        @click="$emit('refreshAgents')"
        :disabled="isLoadingAgents"
        class="button-secondary"
      >
        {{ isLoadingAgents ? "Refreshing Agents..." : "Refresh Agent List" }}
      </button>
      <button
        @click="$emit('refreshMetrics')"
        :disabled="!selectedInstanceId || isLoadingMetrics"
        v-if="selectedInstanceId"
        class="button-primary"
      >
        {{ isLoadingMetrics ? "Loading Metrics..." : "Refresh Metrics Data" }}
      </button>
    </div>
    <p v-if="isLoadingAgents">Loading available agents...</p>
    <p v-if="!isLoadingAgents && availableAgents.length === 0">
      No agents found. Ensure agents are registered with the API.
    </p>
  </section>
</template>

<script setup>
defineProps({
  availableAgents: {
    type: Array,
    required: true,
  },
  selectedInstanceId: {
    type: String,
    required: true,
  },
  isLoadingAgents: {
    type: Boolean,
    required: true,
  },
  isLoadingMetrics: {
    type: Boolean,
    required: true,
  },
});

const emit = defineEmits(['update:selectedInstanceId', 'refreshAgents', 'refreshMetrics']);

function handleSelectionChange(event) {
  emit('update:selectedInstanceId', event.target.value);
}
</script>

<style scoped>
.agent-selection {
  background-color: #ffffff;
  padding: 1.5rem;
  border-radius: 0.5rem;
  box-shadow:
    0 1px 3px 0 rgba(0, 0, 0, 0.1),
    0 1px 2px -1px rgba(0, 0, 0, 0.1);
  margin-bottom: 2rem;
}

.agent-selection h2 {
  font-size: 1.25rem;
  font-weight: 600;
  color: #111827;
  margin-top: 0;
  margin-bottom: 1rem;
}

.controls {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  align-items: center;
  margin-bottom: 0.5rem;
}

.agent-selection select {
  padding: 0.625rem 0.875rem;
  border: 1px solid #d1d5db;
  border-radius: 0.375rem;
  font-size: 0.875rem;
  flex-grow: 1;
  min-width: 250px;
  background-color: #fff;
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
  background-color: #3b82f6;
}

.button-primary:hover {
  background-color: #2563eb;
}

.button-secondary {
  background-color: #6b7280;
}

.button-secondary:hover {
  background-color: #4b5563;
}

.agent-selection button:disabled {
  background-color: #e5e7eb;
  color: #9ca3af;
  cursor: not-allowed;
}

.agent-selection p {
  font-size: 0.875rem;
  color: #4b5563;
}
</style>