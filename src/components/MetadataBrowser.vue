<template>
  <div class="metadata-browser">
    <div class="header">
      <h2>Metadata Browser</h2>
      <div class="header-actions">
        <select v-model="selectedDataSourceId" @change="loadMetadata">
          <option value="">Select Data Source</option>
          <option v-for="ds in dataSources" :key="ds.id" :value="ds.id">
            {{ ds.name }} ({{ ds.data_type }})
          </option>
        </select>
        <button v-if="selectedDataSourceId" @click="refreshMetadata" class="btn-primary">Refresh</button>
      </div>
    </div>

    <div v-if="error" class="error">{{ error }}</div>

    <div v-if="loading" class="loading">Loading...</div>

    <div v-else-if="selectedDataSource" class="metadata-content">
      <!-- Database Tables -->
      <div v-if="selectedDataSource.data_type !== 'kafka'">
        <h3>Tables</h3>
        <div class="table-list">
          <div
            v-for="table in tables"
            :key="table.name"
            class="table-item"
            @click="selectTable(table)"
            :class="{ active: selectedTable?.name === table.name }"
          >
            <div class="table-info">
              <strong>{{ table.name }}</strong>
              <span v-if="table.schema" class="schema">{{ table.schema }}</span>
              <span v-if="table.row_count !== undefined" class="row-count">
                {{ table.row_count.toLocaleString() }} rows
              </span>
            </div>
          </div>
        </div>

        <!-- Table Structure -->
        <div v-if="selectedTable" class="table-structure">
          <h3>Table Structure: {{ selectedTable.name }}</h3>
          <table>
            <thead>
              <tr>
                <th>Column</th>
                <th>Type</th>
                <th>Nullable</th>
                <th>Default</th>
                <th>Constraints</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="col in selectedTable.columns" :key="col.name">
                <td>{{ col.name }}</td>
                <td>{{ col.data_type }}</td>
                <td>{{ col.is_nullable ? 'YES' : 'NO' }}</td>
                <td>{{ col.default_value || '-' }}</td>
                <td>{{ col.constraints.join(', ') || '-' }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- Kafka Topics -->
      <div v-else>
        <h3>Topics</h3>
        <div class="topic-list">
          <div
            v-for="topic in kafkaTopics"
            :key="topic.name"
            class="topic-item"
          >
            <div class="topic-info">
              <strong>{{ topic.name }}</strong>
              <span class="partition-count">{{ topic.partitions.length }} partitions</span>
            </div>
            <div class="partitions">
              <div
                v-for="partition in topic.partitions"
                :key="partition.id"
                class="partition-item"
              >
                Partition {{ partition.id }}: Leader={{ partition.leader }},
                Replicas=[{{ partition.replicas.join(',') }}],
                ISR=[{{ partition.isr.join(',') }}]
              </div>
            </div>
          </div>
        </div>

        <!-- Schema Registry -->
        <div v-if="schemas.length > 0" class="schemas">
          <h3>Schema Registry Schemas</h3>
          <div class="schema-list">
            <div
              v-for="schema in schemas"
              :key="schema.subject"
              class="schema-item"
            >
              <div class="schema-info">
                <strong>{{ schema.subject }}</strong>
                <span class="schema-version">v{{ schema.version }}</span>
                <span class="schema-type">{{ schema.schema_type }}</span>
              </div>
              <pre class="schema-content">{{ schema.schema }}</pre>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useDataSources } from '../composables/useDataSources';
import type { DataSource, TableInfo, KafkaTopicInfo, SchemaInfo } from '../types';

const {
  dataSources,
  loading,
  error,
  loadDataSources,
  getTables,
  getTableStructure,
  getKafkaTopics,
  getSchemaRegistrySchemas,
  refreshMetadata: refreshMeta,
} = useDataSources();

onMounted(() => {
  loadDataSources();
});

const selectedDataSourceId = ref<number | null>(null);
const tables = ref<TableInfo[]>([]);
const selectedTable = ref<TableInfo | null>(null);
const kafkaTopics = ref<KafkaTopicInfo[]>([]);
const schemas = ref<SchemaInfo[]>([]);

const selectedDataSource = computed(() => {
  if (!selectedDataSourceId.value) return null;
  return dataSources.value.find(ds => ds.id === selectedDataSourceId.value) || null;
});

const loadMetadata = async () => {
  if (!selectedDataSourceId.value) return;
  
  const ds = selectedDataSource.value;
  if (!ds) return;

  try {
    if (ds.data_type === 'kafka') {
      kafkaTopics.value = await getKafkaTopics(selectedDataSourceId.value, false);
      if (ds.schema_registry_url) {
        schemas.value = await getSchemaRegistrySchemas(selectedDataSourceId.value, false);
      }
    } else {
      tables.value = await getTables(selectedDataSourceId.value, false);
    }
  } catch (e) {
    console.error('Failed to load metadata:', e);
  }
};

const selectTable = async (table: TableInfo) => {
  if (!selectedDataSourceId.value) return;
  
  try {
    const fullTable = await getTableStructure(
      selectedDataSourceId.value,
      table.schema,
      table.name,
      false
    );
    selectedTable.value = fullTable;
  } catch (e) {
    console.error('Failed to load table structure:', e);
  }
};

const refreshMetadata = async () => {
  if (!selectedDataSourceId.value) return;
  
  try {
    await refreshMeta(selectedDataSourceId.value);
    await loadMetadata();
  } catch (e) {
    console.error('Failed to refresh metadata:', e);
  }
};
</script>

<style scoped>
.metadata-browser {
  padding: 20px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.header-actions {
  display: flex;
  gap: 10px;
  align-items: center;
}

.header-actions select {
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
}

.table-list,
.topic-list,
.schema-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 20px;
}

.table-item,
.topic-item,
.schema-item {
  padding: 15px;
  border: 1px solid #ddd;
  border-radius: 8px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.table-item:hover {
  background-color: #f5f5f5;
}

.table-item.active {
  background-color: #e3f2fd;
  border-color: #2196f3;
}

.table-info {
  display: flex;
  gap: 10px;
  align-items: center;
}

.schema {
  color: #666;
  font-size: 0.9em;
}

.row-count {
  color: #4caf50;
  font-size: 0.9em;
}

.table-structure {
  margin-top: 20px;
}

.table-structure table {
  width: 100%;
  border-collapse: collapse;
}

.table-structure th,
.table-structure td {
  padding: 10px;
  text-align: left;
  border: 1px solid #ddd;
}

.table-structure th {
  background-color: #f5f5f5;
  font-weight: bold;
}

.partition-count {
  color: #2196f3;
  font-size: 0.9em;
  margin-left: 10px;
}

.partitions {
  margin-top: 10px;
  padding-left: 20px;
}

.partition-item {
  font-size: 0.9em;
  color: #666;
  margin: 5px 0;
}

.schema-info {
  display: flex;
  gap: 10px;
  align-items: center;
  margin-bottom: 10px;
}

.schema-version {
  color: #666;
  font-size: 0.9em;
}

.schema-type {
  color: #ff9800;
  font-size: 0.9em;
}

.schema-content {
  background-color: #f5f5f5;
  padding: 10px;
  border-radius: 4px;
  overflow-x: auto;
  font-size: 0.85em;
}

.error {
  color: red;
  padding: 10px;
  background-color: #ffebee;
  border-radius: 4px;
  margin-bottom: 10px;
}

.loading {
  text-align: center;
  padding: 20px;
}
</style>

