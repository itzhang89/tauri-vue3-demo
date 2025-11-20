<template>
  <div class="table-comparator">
    <div class="header">
      <h2>Table Comparison</h2>
    </div>

    <div class="comparison-form">
      <div class="form-row">
        <div class="form-group">
          <label>Source 1:</label>
          <select v-model="source1Id">
            <option value="">Select data source</option>
            <option v-for="ds in dataSources" :key="ds.id" :value="ds.id">
              {{ ds.name }} ({{ ds.data_type }})
            </option>
          </select>
        </div>
        <div class="form-group">
          <label>Schema 1 (optional):</label>
          <input v-model="schema1" placeholder="e.g., public, dbo" />
        </div>
      </div>
      <div class="form-row">
        <div class="form-group">
          <label>Source 2:</label>
          <select v-model="source2Id">
            <option value="">Select data source</option>
            <option v-for="ds in dataSources" :key="ds.id" :value="ds.id">
              {{ ds.name }} ({{ ds.data_type }})
            </option>
          </select>
        </div>
        <div class="form-group">
          <label>Schema 2 (optional):</label>
          <input v-model="schema2" placeholder="e.g., public, dbo" />
        </div>
      </div>
      <div class="form-row">
        <div class="form-group">
          <label>Table Name:</label>
          <input v-model="tableName" placeholder="Enter table name" />
        </div>
        <button @click="compare" :disabled="!canCompare" class="btn-primary">Compare</button>
      </div>
    </div>

    <div v-if="error" class="error">{{ error }}</div>

    <div v-if="loading" class="loading">Comparing...</div>

    <div v-if="comparison" class="comparison-results">
      <div class="summary">
        <h3>Comparison Summary</h3>
        <div class="summary-item">
          <strong>Table:</strong> {{ comparison.table_name }}
        </div>
        <div class="summary-item">
          <strong>Row Count Difference:</strong>
          <span :class="rowCountDiffClass">
            {{ comparison.row_count_diff !== undefined ? (comparison.row_count_diff > 0 ? '+' : '') + comparison.row_count_diff.toLocaleString() : 'N/A' }}
          </span>
        </div>
        <div class="summary-item">
          <strong>Structure Differences:</strong> {{ comparison.structure_diff.length }}
        </div>
      </div>

      <div class="tables-comparison">
        <div class="table-side">
          <h4>Source 1</h4>
          <div class="table-info">
            <p><strong>Rows:</strong> {{ comparison.source1.row_count?.toLocaleString() || 'N/A' }}</p>
            <p><strong>Columns:</strong> {{ comparison.source1.columns.length }}</p>
          </div>
          <table>
            <thead>
              <tr>
                <th>Column</th>
                <th>Type</th>
                <th>Nullable</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="col in comparison.source1.columns"
                :key="col.name"
                :class="getColumnDiffClass(col.name, 'source1')"
              >
                <td>{{ col.name }}</td>
                <td>{{ col.data_type }}</td>
                <td>{{ col.is_nullable ? 'YES' : 'NO' }}</td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="table-side">
          <h4>Source 2</h4>
          <div class="table-info">
            <p><strong>Rows:</strong> {{ comparison.source2.row_count?.toLocaleString() || 'N/A' }}</p>
            <p><strong>Columns:</strong> {{ comparison.source2.columns.length }}</p>
          </div>
          <table>
            <thead>
              <tr>
                <th>Column</th>
                <th>Type</th>
                <th>Nullable</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="col in comparison.source2.columns"
                :key="col.name"
                :class="getColumnDiffClass(col.name, 'source2')"
              >
                <td>{{ col.name }}</td>
                <td>{{ col.data_type }}</td>
                <td>{{ col.is_nullable ? 'YES' : 'NO' }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div v-if="comparison.structure_diff.length > 0" class="differences">
        <h3>Structure Differences</h3>
        <div
          v-for="(diff, index) in comparison.structure_diff"
          :key="index"
          class="diff-item"
          :class="'diff-' + diff.diff_type"
        >
          <div class="diff-header">
            <strong>{{ diff.column_name }}</strong>
            <span class="diff-type">{{ diff.diff_type.toUpperCase() }}</span>
          </div>
          <div v-if="diff.source1_value" class="diff-value">
            <strong>Source 1:</strong> {{ diff.source1_value }}
          </div>
          <div v-if="diff.source2_value" class="diff-value">
            <strong>Source 2:</strong> {{ diff.source2_value }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useDataSources } from '../composables/useDataSources';
import type { TableComparison } from '../types';

const {
  dataSources,
  loading,
  error,
  loadDataSources,
  compareTables,
} = useDataSources();

onMounted(() => {
  loadDataSources();
});

const source1Id = ref<number | null>(null);
const source2Id = ref<number | null>(null);
const schema1 = ref('');
const schema2 = ref('');
const tableName = ref('');
const comparison = ref<TableComparison | null>(null);

const canCompare = computed(() => {
  return source1Id.value && source2Id.value && tableName.value.trim() !== '';
});

const rowCountDiffClass = computed(() => {
  if (!comparison.value || comparison.value.row_count_diff === undefined) return '';
  if (comparison.value.row_count_diff > 0) return 'diff-positive';
  if (comparison.value.row_count_diff < 0) return 'diff-negative';
  return 'diff-neutral';
});

const compare = async () => {
  if (!canCompare.value) return;
  
  try {
    comparison.value = await compareTables(
      source1Id.value!,
      source2Id.value!,
      schema1.value || undefined,
      schema2.value || undefined,
      tableName.value
    );
  } catch (e) {
    console.error('Failed to compare tables:', e);
  }
};

const getColumnDiffClass = (columnName: string, source: 'source1' | 'source2') => {
  if (!comparison.value) return '';
  
  const diff = comparison.value.structure_diff.find(d => d.column_name === columnName);
  if (!diff) return '';
  
  if (diff.diff_type === 'added' && source === 'source2') return 'diff-added';
  if (diff.diff_type === 'removed' && source === 'source1') return 'diff-removed';
  if (diff.diff_type === 'modified') return 'diff-modified';
  
  return '';
};
</script>

<style scoped>
.table-comparator {
  padding: 20px;
}

.comparison-form {
  background-color: #f5f5f5;
  padding: 20px;
  border-radius: 8px;
  margin-bottom: 20px;
}

.form-row {
  display: flex;
  gap: 15px;
  margin-bottom: 15px;
  align-items: flex-end;
}

.form-group {
  flex: 1;
}

.form-group label {
  display: block;
  margin-bottom: 5px;
  font-weight: bold;
}

.form-group select,
.form-group input {
  width: 100%;
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
  box-sizing: border-box;
}

.comparison-results {
  margin-top: 20px;
}

.summary {
  background-color: #e3f2fd;
  padding: 15px;
  border-radius: 8px;
  margin-bottom: 20px;
}

.summary-item {
  margin: 10px 0;
}

.diff-positive {
  color: #4caf50;
}

.diff-negative {
  color: #f44336;
}

.diff-neutral {
  color: #666;
}

.tables-comparison {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
  margin-bottom: 20px;
}

.table-side {
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 15px;
}

.table-side h4 {
  margin-top: 0;
}

.table-info {
  margin-bottom: 15px;
  padding: 10px;
  background-color: #f5f5f5;
  border-radius: 4px;
}

.table-side table {
  width: 100%;
  border-collapse: collapse;
}

.table-side th,
.table-side td {
  padding: 8px;
  text-align: left;
  border: 1px solid #ddd;
}

.table-side th {
  background-color: #f5f5f5;
  font-weight: bold;
}

.diff-added {
  background-color: #c8e6c9;
}

.diff-removed {
  background-color: #ffcdd2;
}

.diff-modified {
  background-color: #fff9c4;
}

.differences {
  margin-top: 20px;
}

.diff-item {
  padding: 15px;
  border: 1px solid #ddd;
  border-radius: 8px;
  margin-bottom: 10px;
}

.diff-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.diff-type {
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 0.8em;
  font-weight: bold;
}

.diff-added .diff-type {
  background-color: #4caf50;
  color: white;
}

.diff-removed .diff-type {
  background-color: #f44336;
  color: white;
}

.diff-modified .diff-type {
  background-color: #ff9800;
  color: white;
}

.diff-value {
  margin: 5px 0;
  padding: 5px;
  background-color: #f5f5f5;
  border-radius: 4px;
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

