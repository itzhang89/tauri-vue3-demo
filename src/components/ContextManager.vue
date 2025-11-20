<template>
  <div class="context-manager">
    <div class="header">
      <h2>Contexts</h2>
      <button @click="showCreateDialog = true" class="btn-primary">Create Context</button>
    </div>

    <div v-if="error" class="error">{{ error }}</div>

    <div v-if="loading" class="loading">Loading...</div>

    <div v-else class="context-list">
      <div
        v-for="context in contexts"
        :key="context.id"
        class="context-item"
        @click="selectContext(context)"
        :class="{ active: selectedContext?.id === context.id }"
      >
        <div class="context-info">
          <h3>{{ context.name }}</h3>
          <p v-if="context.description">{{ context.description }}</p>
        </div>
        <div class="context-actions">
          <button @click.stop="editContext(context)" class="btn-icon">✏️</button>
          <button @click.stop="handleDelete(context.id)" class="btn-icon">🗑️</button>
        </div>
      </div>
    </div>

    <!-- Create/Edit Dialog -->
    <div v-if="showCreateDialog || editingContext" class="dialog-overlay" @click="closeDialog">
      <div class="dialog" @click.stop>
        <h3>{{ editingContext ? 'Edit Context' : 'Create Context' }}</h3>
        <form @submit.prevent="saveContext">
          <div class="form-group">
            <label>Name:</label>
            <input v-model="formData.name" required />
          </div>
          <div class="form-group">
            <label>Description:</label>
            <textarea v-model="formData.description" rows="3"></textarea>
          </div>
          <div class="form-actions">
            <button type="submit" class="btn-primary">Save</button>
            <button type="button" @click="closeDialog" class="btn-secondary">Cancel</button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useDataSources } from '../composables/useDataSources';
import type { Context } from '../types';

const {
  contexts,
  loading,
  error,
  loadContexts,
  createContext,
  updateContext,
  deleteContext,
} = useDataSources();

const selectedContext = ref<Context | null>(null);
const showCreateDialog = ref(false);
const editingContext = ref<Context | null>(null);
const formData = ref({ name: '', description: '' });

const emit = defineEmits<{
  (e: 'select', context: Context): void;
}>();

onMounted(() => {
  loadContexts();
});

const selectContext = (context: Context) => {
  selectedContext.value = context;
  emit('select', context);
};

const editContext = (context: Context) => {
  editingContext.value = context;
  formData.value = {
    name: context.name,
    description: context.description || '',
  };
};

const saveContext = async () => {
  try {
    if (editingContext.value) {
      await updateContext(editingContext.value.id, formData.value.name, formData.value.description || undefined);
    } else {
      await createContext(formData.value.name, formData.value.description || undefined);
    }
    closeDialog();
  } catch (e) {
    console.error('Failed to save context:', e);
  }
};

const closeDialog = () => {
  showCreateDialog.value = false;
  editingContext.value = null;
  formData.value = { name: '', description: '' };
};

const handleDelete = async (id: number) => {
  if (confirm('Are you sure you want to delete this context?')) {
    try {
      await deleteContext(id);
      if (selectedContext.value?.id === id) {
        selectedContext.value = null;
      }
    } catch (e) {
      console.error('Failed to delete context:', e);
    }
  }
};

defineExpose({ selectedContext });
</script>

<style scoped>
.context-manager {
  padding: 20px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.context-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.context-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px;
  border: 1px solid #ddd;
  border-radius: 8px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.context-item:hover {
  background-color: #f5f5f5;
}

.context-item.active {
  background-color: #e3f2fd;
  border-color: #2196f3;
}

.context-info h3 {
  margin: 0 0 5px 0;
}

.context-info p {
  margin: 0;
  color: #666;
  font-size: 0.9em;
}

.context-actions {
  display: flex;
  gap: 10px;
}

.btn-primary {
  background-color: #2196f3;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
}

.btn-icon {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 1.2em;
  padding: 5px;
}

.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.dialog {
  background: white;
  padding: 20px;
  border-radius: 8px;
  min-width: 400px;
}

.form-group {
  margin-bottom: 15px;
}

.form-group label {
  display: block;
  margin-bottom: 5px;
  font-weight: bold;
}

.form-group input,
.form-group textarea {
  width: 100%;
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
  box-sizing: border-box;
}

.form-actions {
  display: flex;
  gap: 10px;
  justify-content: flex-end;
}

.btn-secondary {
  background-color: #ccc;
  color: black;
  border: none;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
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

