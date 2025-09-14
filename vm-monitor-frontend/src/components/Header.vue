<template>
  <header class="header">
    <h1>VM Monitor Dashboard</h1>
    <p
      class="api-status"
      :class="{ connected: isApiConnected, disconnected: !isApiConnected }"
    >
      API Status: {{ apiStatusMessage }}
    </p>
  </header>
</template>

<script setup>
defineProps({
  isApiConnected: {
    type: Boolean,
    required: true,
  },
  apiStatusMessage: {
    type: String,
    required: true,
  },
});
</script>

<style scoped>
.header {
  border-bottom: 1px solid #d1d5db; /* Tailwind gray-300 */
  padding-bottom: 1rem;
  margin-bottom: 2rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header h1 {
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
</style>