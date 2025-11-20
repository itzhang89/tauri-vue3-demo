export interface Context {
  id: number;
  name: string;
  description?: string;
  created_at: string;
  updated_at: string;
}

export interface DataSource {
  id: number;
  context_id: number;
  name: string;
  data_type: 'mysql' | 'postgresql' | 'sqlserver' | 'kafka';
  host: string;
  port: number;
  database?: string;
  username: string;
  password: string;
  proxy_type?: 'socks5' | 'http' | 'ssh';
  proxy_config?: any;
  ssh_config?: any;
  schema_registry_url?: string;
  created_at: string;
  updated_at: string;
}

export interface CreateDataSourceRequest {
  context_id: number;
  name: string;
  data_type: 'mysql' | 'postgresql' | 'sqlserver' | 'kafka';
  host: string;
  port: number;
  database?: string;
  username: string;
  password: string;
  proxy_type?: 'socks5' | 'http' | 'ssh';
  proxy_config?: any;
  ssh_config?: any;
  schema_registry_url?: string;
}

export interface TableInfo {
  name: string;
  schema?: string;
  row_count?: number;
  columns: ColumnInfo[];
}

export interface ColumnInfo {
  name: string;
  data_type: string;
  is_nullable: boolean;
  default_value?: string;
  constraints: string[];
}

export interface TableComparison {
  table_name: string;
  source1: TableInfo;
  source2: TableInfo;
  structure_diff: StructureDiff[];
  row_count_diff?: number;
}

export interface StructureDiff {
  column_name: string;
  diff_type: 'added' | 'removed' | 'modified';
  source1_value?: string;
  source2_value?: string;
}

export interface KafkaTopicInfo {
  name: string;
  partitions: PartitionInfo[];
  consumer_groups: string[];
}

export interface PartitionInfo {
  id: number;
  leader: number;
  replicas: number[];
  isr: number[];
}

export interface SchemaInfo {
  subject: string;
  version: number;
  schema_type: string;
  schema: string;
}

