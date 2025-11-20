<template>
  <div class="data-source-manager">
    <div class="header">
      <h2>Data Sources</h2>
      <div class="header-actions">
        <button @click="showImportDialog = true" class="btn-secondary">Import YAML</button>
        <button @click="showCreateDialog = true" class="btn-primary">Add Data Source</button>
      </div>
    </div>

    <div v-if="error" class="error">{{ error }}</div>

    <div v-if="loading && !dataSources.length" class="loading">Loading...</div>

    <div v-else class="data-source-list">
      <div
        v-for="ds in dataSources"
        :key="ds.id"
        class="data-source-item"
      >
        <div class="ds-info">
          <h3>{{ ds.name }}</h3>
          <p class="ds-type">{{ ds.data_type.toUpperCase() }}</p>
          <p class="ds-connection">{{ ds.host }}:{{ ds.port }}{{ ds.database ? '/' + ds.database : '' }}</p>
          <p v-if="ds.proxy_type" class="ds-proxy">Proxy: {{ ds.proxy_type }}</p>
        </div>
        <div class="ds-actions">
          <button @click="testConnection(ds)" class="btn-test">Test</button>
          <button @click="editDataSource(ds)" class="btn-icon">✏️</button>
          <button @click="deleteDataSource(ds.id)" class="btn-icon">🗑️</button>
        </div>
      </div>
    </div>

    <!-- Create/Edit Dialog -->
    <div v-if="showCreateDialog || editingDataSource" class="dialog-overlay" @click="closeDialog">
      <div class="dialog" @click.stop>
        <h3>{{ editingDataSource ? 'Edit Data Source' : 'Create Data Source' }}</h3>
        <form @submit.prevent="saveDataSource">
          <div class="form-group">
            <label>Context:</label>
            <select v-model="formData.context_id" required>
              <option v-for="ctx in contexts" :key="ctx.id" :value="ctx.id">{{ ctx.name }}</option>
            </select>
          </div>
          <div class="form-group">
            <label>Name:</label>
            <input v-model="formData.name" required />
          </div>
          <div class="form-group">
            <label>Type:</label>
            <select v-model="formData.data_type" required>
              <option value="mysql">MySQL</option>
              <option value="postgresql">PostgreSQL</option>
              <option value="sqlserver">SQL Server</option>
              <option value="kafka">Kafka</option>
            </select>
          </div>
          <div class="form-group">
            <label>Host:</label>
            <input v-model="formData.host" required />
          </div>
          <div class="form-group">
            <label>Port:</label>
            <input v-model.number="formData.port" type="number" required />
          </div>
          <div class="form-group" v-if="formData.data_type !== 'kafka'">
            <label>Database:</label>
            <input v-model="formData.database" />
          </div>
          <div class="form-group">
            <label>Username:</label>
            <input v-model="formData.username" required />
          </div>
          <div class="form-group">
            <label>Password:</label>
            <input v-model="formData.password" type="password" required />
          </div>
          <div class="form-group" v-if="formData.data_type === 'kafka'">
            <label>Schema Registry URL:</label>
            <input v-model="formData.schema_registry_url" />
          </div>
          <div class="form-group">
            <label>Proxy Type:</label>
            <select v-model="formData.proxy_type">
              <option value="">None</option>
              <option value="socks5">SOCKS5</option>
              <option value="http">HTTP</option>
              <option value="ssh">SSH Tunnel</option>
            </select>
          </div>
          <div v-if="formData.proxy_type" class="proxy-config">
            <h4>Proxy Configuration</h4>
            <div class="form-group" v-if="formData.proxy_type === 'socks5' || formData.proxy_type === 'http'">
              <label>Proxy Host:</label>
              <input v-model="proxyConfig.host" />
            </div>
            <div class="form-group" v-if="formData.proxy_type === 'socks5' || formData.proxy_type === 'http'">
              <label>Proxy Port:</label>
              <input v-model.number="proxyConfig.port" type="number" />
            </div>
            <div class="form-group" v-if="formData.proxy_type === 'ssh'">
              <label>SSH Host:</label>
              <input v-model="sshConfig.host" />
            </div>
            <div class="form-group" v-if="formData.proxy_type === 'ssh'">
              <label>SSH Port:</label>
              <input v-model.number="sshConfig.port" type="number" />
            </div>
            <div class="form-group" v-if="formData.proxy_type === 'ssh'">
              <label>SSH Username:</label>
              <input v-model="sshConfig.username" />
            </div>
            <div class="form-group" v-if="formData.proxy_type === 'ssh'">
              <label>SSH Password:</label>
              <input v-model="sshConfig.password" type="password" />
            </div>
          </div>
          <div class="form-actions">
            <button type="submit" class="btn-primary">Save</button>
            <button type="button" @click="closeDialog" class="btn-secondary">Cancel</button>
          </div>
        </form>
      </div>
    </div>

    <!-- Import Dialog -->
    <div v-if="showImportDialog" class="dialog-overlay" @click="closeImportDialog">
      <div class="dialog" @click.stop>
        <h3>Import Data Sources from YAML</h3>
        <form @submit.prevent="importYAML">
          <div class="form-group">
            <label>Context:</label>
            <select v-model="importContextId" required>
              <option v-for="ctx in contexts" :key="ctx.id" :value="ctx.id">{{ ctx.name }}</option>
            </select>
          </div>
          <div class="form-group">
            <label>YAML File Path:</label>
            <input v-model="yamlFilePath" required />
            <button type="button" @click="selectFile" class="btn-secondary">Browse</button>
          </div>
          <div class="form-actions">
            <button type="submit" class="btn-primary">Import</button>
            <button type="button" @click="closeImportDialog" class="btn-secondary">Cancel</button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-opener';
import { useDataSources } from '../composables/useDataSources';
import type { DataSource, CreateDataSourceRequest, Context } from '../types';

const props = defineProps<{
  contextId?: number;
}>();

const {
  contexts,
  dataSources,
  loading,
  error,
  loadContexts,
  loadDataSources,
  createDataSource,
  testConnection: testConn,
} = useDataSources();

const showCreateDialog = ref(false);
const showImportDialog = ref(false);
const editingDataSource = ref<DataSource | null>(null);
const formData = ref<CreateDataSourceRequest>({
  context_id: 0,
  name: '',
  data_type: 'mysql',
  host: '',
  port: 3306,
  database: '',
  username: '',
  password: '',
  proxy_type: undefined,
  proxy_config: undefined,
  ssh_config: undefined,
  schema_registry_url: undefined,
});

const proxyConfig = ref({ host: '', port: 1080, username: '', password: '' });
const sshConfig = ref({ host: '', port: 22, username: '', password: '', local_port: 3306 });
const importContextId = ref<number | null>(null);
const yamlFilePath = ref('');

onMounted(async () => {
  await loadContexts();
  if (props.contextId) {
    await loadDataSources(props.contextId);
  } else {
    await loadDataSources();
  }
});

watch(() => props.contextId, async (newId) => {
  if (newId) {
    await loadDataSources(newId);
  } else {
    await loadDataSources();
  }
});

const editDataSource = (ds: DataSource) => {
  editingDataSource.value = ds;
  formData.value = {
    context_id: ds.context_id,
    name: ds.name,
    data_type: ds.data_type,
    host: ds.host,
    port: ds.port,
    database: ds.database,
    username: ds.username,
    password: ds.password,
    proxy_type: ds.proxy_type,
    proxy_config: ds.proxy_config,
    ssh_config: ds.ssh_config,
    schema_registry_url: ds.schema_registry_url,
  };
  if (ds.proxy_config) {
    proxyConfig.value = { ...ds.proxy_config } as any;
  }
  if (ds.ssh_config) {
    sshConfig.value = { ...ds.ssh_config } as any;
  }
};

const saveDataSource = async () => {
  try {
    const req: CreateDataSourceRequest = {
      ...formData.value,
      proxy_config: formData.value.proxy_type ? {
        proxy_type: formData.value.proxy_type,
        ...proxyConfig.value,
      } : undefined,
      ssh_config: formData.value.proxy_type === 'ssh' ? sshConfig.value : undefined,
    };
    await createDataSource(req);
    closeDialog();
  } catch (e) {
    console.error('Failed to save data source:', e);
  }
};

const testConnection = async (ds: DataSource) => {
  try {
    await testConn(ds);
    alert('Connection successful!');
  } catch (e: any) {
    alert('Connection failed: ' + e.toString());
  }
};

const deleteDataSource = async (id: number) => {
  if (confirm('Are you sure you want to delete this data source?')) {
    try {
      await invoke('delete_data_source', { id });
      await loadDataSources(props.contextId);
    } catch (e) {
      console.error('Failed to delete data source:', e);
    }
  }
};

const selectFile = async () => {
  // In a real implementation, use Tauri's file dialog
  // For now, just a placeholder
  yamlFilePath.value = prompt('Enter file path:') || '';
};

const importYAML = async () => {
  if (!importContextId.value || !yamlFilePath.value) {
    alert('Please select context and file path');
    return;
  }
  try {
    await invoke('import_data_sources_from_yaml', {
      filePath: yamlFilePath.value,
      contextId: importContextId.value,
    });
    await loadDataSources(importContextId.value);
    closeImportDialog();
    alert('Import successful!');
  } catch (e: any) {
    alert('Import failed: ' + e.toString());
  }
};

const closeDialog = () => {
  showCreateDialog.value = false;
  editingDataSource.value = null;
  formData.value = {
    context_id: contexts.value[0]?.id || 0,
    name: '',
    data_type: 'mysql',
    host: '',
    port: 3306,
    database: '',
    username: '',
    password: '',
    proxy_type: undefined,
    proxy_config: undefined,
    ssh_config: undefined,
    schema_registry_url: undefined,
  };
  proxyConfig.value = { host: '', port: 1080, username: '', password: '' };
  sshConfig.value = { host: '', port: 22, username: '', password: '', local_port: 3306 };
};

const closeImportDialog = () => {
  showImportDialog.value = false;
  importContextId.value = null;
  yamlFilePath.value = '';
};
</script>

<style scoped>
.data-source-manager {
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
}

.data-source-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 15px;
}

.data-source-item {
  padding: 15px;
  border: 1px solid #ddd;
  border-radius: 8px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.ds-info h3 {
  margin: 0 0 5px 0;
}

.ds-type {
  font-weight: bold;
  color: #2196f3;
  margin: 5px 0;
}

.ds-connection {
  color: #666;
  font-size: 0.9em;
  margin: 5px 0;
}

.ds-proxy {
  color: #ff9800;
  font-size: 0.85em;
  margin: 5px 0;
}

.ds-actions {
  display: flex;
  gap: 5px;
}

.btn-test {
  background-color: #4caf50;
  color: white;
  border: none;
  padding: 5px 10px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.9em;
}

.proxy-config {
  margin-top: 15px;
  padding: 15px;
  background-color: #f5f5f5;
  border-radius: 4px;
}

.proxy-config h4 {
  margin-top: 0;
}

.dialog {
  max-height: 90vh;
  overflow-y: auto;
}

/* Reuse styles from ContextManager */
</style>

