import type { ActorSubclass } from '@dfinity/agent';
import type { _SERVICE } from '../declarations/agent/agent.did';

export type { _SERVICE };

export type { ActorSubclass as AgentActor };

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

export interface ToolResult {
  name: string;
  success: boolean;
  output: string;
  cost_cycles: bigint;
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

export interface ScheduledTask {
  id: string;
  cron_expr: string;
  payload: string;
  next_run: bigint;
  enabled: boolean;
}