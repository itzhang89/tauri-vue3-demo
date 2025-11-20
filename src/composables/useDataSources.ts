import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Context, DataSource, CreateDataSourceRequest, TableInfo, KafkaTopicInfo, SchemaInfo, TableComparison } from '../types';

export function useDataSources() {
  const contexts = ref<Context[]>([]);
  const dataSources = ref<DataSource[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  const loadContexts = async () => {
    try {
      loading.value = true;
      error.value = null;
      contexts.value = await invoke<Context[]>('list_contexts');
    } catch (e: any) {
      error.value = e.toString();
      console.error('Failed to load contexts:', e);
    } finally {
      loading.value = false;
    }
  };

  const createContext = async (name: string, description?: string) => {
    try {
      loading.value = true;
      error.value = null;
      await invoke('create_context', { name, description });
      await loadContexts();
    } catch (e: any) {
      error.value = e.toString();
      throw e;
    } finally {
      loading.value = false;
    }
  };

  const updateContext = async (id: number, name: string, description?: string) => {
    try {
      loading.value = true;
      error.value = null;
      await invoke('update_context', { id, name, description });
      await loadContexts();
    } catch (e: any) {
      error.value = e.toString();
      throw e;
    } finally {
      loading.value = false;
    }
  };

  const deleteContext = async (id: number) => {
    try {
      loading.value = true;
      error.value = null;
      await invoke('delete_context', { id });
      await loadContexts();
    } catch (e: any) {
      error.value = e.toString();
      throw e;
    } finally {
      loading.value = false;
    }
  };

  const loadDataSources = async (contextId?: number) => {
    try {
      loading.value = true;
      error.value = null;
      dataSources.value = await invoke<DataSource[]>('list_data_sources', { contextId });
    } catch (e: any) {
      error.value = e.toString();
      console.error('Failed to load data sources:', e);
    } finally {
      loading.value = false;
    }
  };

  const createDataSource = async (req: CreateDataSourceRequest) => {
    try {
      loading.value = true;
      error.value = null;
      await invoke('create_data_source', { req });
      await loadDataSources(req.context_id);
    } catch (e: any) {
      error.value = e.toString();
      throw e;
    } finally {
      loading.value = false;
    }
  };

  const testConnection = async (dataSource: DataSource) => {
    try {
      loading.value = true;
      error.value = null;
      await invoke('test_connection', { dataSource });
      return true;
    } catch (e: any) {
      error.value = e.toString();
      throw e;
    } finally {
      loading.value = false;
    }
  };

  const getTables = async (dataSourceId: number, forceRefresh = false): Promise<TableInfo[]> => {
    try {
      loading.value = true;
      error.value = null;
      return await invoke<TableInfo[]>('get_tables', { dataSourceId, forceRefresh });
    } catch (e: any) {
      error.value = e.toString();
      throw e;
    } finally {
      loading.value = false;
    }
  };

  const getTableStructure = async (
    dataSourceId: number,
    schema: string | undefined,
    tableName: string,
    forceRefresh = false
  ): Promise<TableInfo> => {
    try {
      loading.value = true;
      error.value = null;
      return await invoke<TableInfo>('get_table_structure', {
        dataSourceId,
        schema,
        tableName,
        forceRefresh,
      });
    } catch (e: any) {
      error.value = e.toString();
      throw e;
    } finally {
      loading.value = false;
    }
  };

  const getKafkaTopics = async (dataSourceId: number, forceRefresh = false): Promise<KafkaTopicInfo[]> => {
    try {
      loading.value = true;
      error.value = null;
      return await invoke<KafkaTopicInfo[]>('get_kafka_topics', { dataSourceId, forceRefresh });
    } catch (e: any) {
      error.value = e.toString();
      throw e;
    } finally {
      loading.value = false;
    }
  };

  const getSchemaRegistrySchemas = async (dataSourceId: number, forceRefresh = false): Promise<SchemaInfo[]> => {
    try {
      loading.value = true;
      error.value = null;
      return await invoke<SchemaInfo[]>('get_schema_registry_schemas', { dataSourceId, forceRefresh });
    } catch (e: any) {
      error.value = e.toString();
      throw e;
    } finally {
      loading.value = false;
    }
  };

  const refreshMetadata = async (dataSourceId: number, cacheType?: string) => {
    try {
      loading.value = true;
      error.value = null;
      await invoke('refresh_metadata', { dataSourceId, cacheType });
    } catch (e: any) {
      error.value = e.toString();
      throw e;
    } finally {
      loading.value = false;
    }
  };

  const compareTables = async (
    source1Id: number,
    source2Id: number,
    schema1: string | undefined,
    schema2: string | undefined,
    tableName: string
  ): Promise<TableComparison> => {
    try {
      loading.value = true;
      error.value = null;
      return await invoke<TableComparison>('compare_tables', {
        source1Id,
        source2Id,
        schema1,
        schema2,
        tableName,
      });
    } catch (e: any) {
      error.value = e.toString();
      throw e;
    } finally {
      loading.value = false;
    }
  };

  return {
    contexts,
    dataSources,
    loading,
    error,
    loadContexts,
    createContext,
    updateContext,
    deleteContext,
    loadDataSources,
    createDataSource,
    testConnection,
    getTables,
    getTableStructure,
    getKafkaTopics,
    getSchemaRegistrySchemas,
    refreshMetadata,
    compareTables,
  };
}

