<template>
  <div class="app">
    <div class="sidebar">
      <h1>Data Explorer</h1>
      <nav>
        <button
          v-for="tab in tabs"
          :key="tab.id"
          @click="activeTab = tab.id"
          :class="{ active: activeTab === tab.id }"
          class="nav-button"
        >
          {{ tab.label }}
        </button>
      </nav>
    </div>
    <div class="main-content">
      <ContextManager
        v-if="activeTab === 'contexts'"
        @select="handleContextSelect"
      />
      <DataSourceManager
        v-if="activeTab === 'datasources'"
        :context-id="selectedContextId"
      />
      <MetadataBrowser
        v-if="activeTab === 'metadata'"
      />
      <TableComparator
        v-if="activeTab === 'comparison'"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import ContextManager from './components/ContextManager.vue';
import DataSourceManager from './components/DataSourceManager.vue';
import MetadataBrowser from './components/MetadataBrowser.vue';
import TableComparator from './components/TableComparator.vue';
import { useDataSources } from './composables/useDataSources';
import type { Context } from './types';

const { loadDataSources } = useDataSources();

const activeTab = ref('contexts');
const selectedContextId = ref<number | null>(null);

const tabs = [
  { id: 'contexts', label: 'Contexts' },
  { id: 'datasources', label: 'Data Sources' },
  { id: 'metadata', label: 'Metadata' },
  { id: 'comparison', label: 'Comparison' },
];

const handleContextSelect = (context: Context) => {
  selectedContextId.value = context.id;
  if (activeTab.value === 'datasources') {
    loadDataSources(context.id);
  }
};
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
}

.app {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

.sidebar {
  width: 200px;
  background-color: #2c3e50;
  color: white;
  padding: 20px;
  display: flex;
  flex-direction: column;
}

.sidebar h1 {
  font-size: 1.5em;
  margin-bottom: 30px;
  color: #ecf0f1;
}

nav {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.nav-button {
  background: none;
  border: none;
  color: #bdc3c7;
  padding: 12px 15px;
  text-align: left;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s;
  font-size: 1em;
}

.nav-button:hover {
  background-color: #34495e;
  color: white;
}

.nav-button.active {
  background-color: #3498db;
  color: white;
}

.main-content {
  flex: 1;
  overflow-y: auto;
  background-color: #f8f9fa;
}

.btn-primary {
  background-color: #2196f3;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.9em;
  transition: background-color 0.2s;
}

.btn-primary:hover {
  background-color: #1976d2;
}

.btn-primary:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.btn-secondary {
  background-color: #6c757d;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.9em;
  transition: background-color 0.2s;
}

.btn-secondary:hover {
  background-color: #5a6268;
}

.error {
  color: #d32f2f;
  padding: 12px;
  background-color: #ffebee;
  border-radius: 4px;
  margin-bottom: 15px;
  border-left: 4px solid #d32f2f;
}

.loading {
  text-align: center;
  padding: 40px;
  color: #666;
}
</style>
