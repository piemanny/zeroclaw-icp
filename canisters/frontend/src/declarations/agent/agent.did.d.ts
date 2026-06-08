import type { Actor, HttpAgent } from '@dfinity/agent';

export interface Message {
  role: string;
  content: string;
  timestamp: bigint;
}

export interface Conversation {
  id: string;
  messages: Message[];
  created_at: bigint;
  updated_at: bigint;
}

export interface AgentResponse {
  Ok: string;
  Err: string;
}

export interface ToolResult {
  name: string;
  success: boolean;
  output: string;
  cost_cycles: bigint;
}

export interface ScheduledTask {
  id: string;
  cron_expr: string;
  payload: string;
  next_run: bigint;
  enabled: boolean;
}

export interface SopEntry {
  id: string;
  cron_expr: string;
  prompt: string;
  last_run: bigint;
  enabled: boolean;
  description: string;
}

export interface EconomicsStats {
  balance: bigint;
  operational_mode: string;
  can_execute_outcall: boolean;
  estimated_cycles_per_day: bigint;
}

export interface _SERVICE {
  chat: (message: string) => Promise<AgentResponse>;
  chat_in_conversation: (conversation_id: string, message: string) => Promise<AgentResponse>;
  get_history: (limit: number) => Promise<Message[]>;
  get_conversation: (conversation_id: string) => Promise<Conversation | null>;
  list_conversations: () => Promise<string[]>;
  conversation_count: () => Promise<number>;
  cycles_balance: () => Promise<bigint>;
  get_economics_stats: () => Promise<EconomicsStats>;
  operational_mode: () => Promise<string>;
  set_default_conversation: (conversation_id: string) => Promise<{ Ok: null } | { Err: string }>;
  clear_history: (conversation_id: string) => Promise<{ Ok: null } | { Err: string }>;
  add_sop: (id: string, cron_expr: string, prompt: string, description: string) => Promise<{ Ok: null } | { Err: string }>;
  remove_sop: (id: string) => Promise<{ Ok: null } | { Err: string }>;
  list_sops: () => Promise<SopEntry[]>;
  sop_count: () => Promise<bigint>;
  set_sop_enabled: (id: string, enabled: boolean) => Promise<{ Ok: null } | { Err: string }>;
  recall: (key: string) => Promise<ToolResult>;
  store: (key: string, value: string) => Promise<ToolResult>;
  forget: (key: string) => Promise<ToolResult>;
  list_memory_keys: () => Promise<string[]>;
  schedule_task: (id: string, cron_expr: string, payload: string) => Promise<ToolResult>;
  cancel_task: (id: string) => Promise<ToolResult>;
  list_tasks: () => Promise<ScheduledTask[]>;
  send_notification: (message: string) => Promise<ToolResult>;
  set_owner_principal: (principal: Actor) => Promise<{ Ok: null } | { Err: string }>;
  whoami: () => Promise<Actor>;
  get_transform_func_name: () => Promise<string>;
  handle_notification: (payload: Uint8Array) => Promise<{ Ok: null } | { Err: string }>;
}