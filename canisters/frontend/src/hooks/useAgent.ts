import { useState, useCallback } from 'react';
import { HttpAgent, Actor, ActorSubclass } from '@dfinity/agent';
import { Principal } from '@dfinity/principal';

const AGENT_CANISTER_ID = import.meta.env.VITE_AGENT_CANISTER_ID || 'ryjl3-tyaaa-aaaaa-aaaba-cai';

export interface Message {
  role: string;
  content: string;
  timestamp: number;
}

export interface Conversation {
  id: string;
  messages: Message[];
  created_at: number;
  updated_at: number;
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
  balance: number;
  operational_mode: string;
  can_execute_outcall: boolean;
  estimated_cycles_per_day: number;
}

export interface ScheduledTask {
  id: string;
  cron_expr: string;
  payload: string;
  next_run: bigint;
  enabled: boolean;
}

interface AgentService {
  chat(message: string): Promise<{ Ok: string } | { Err: string }>;
  chat_in_conversation(conversation_id: string, message: string): Promise<{ Ok: string } | { Err: string }>;
  get_history(limit: number): Promise<Message[]>;
  get_conversation(conversation_id: string): Promise<Conversation | null>;
  list_conversations(): Promise<string[]>;
  conversation_count(): Promise<number>;
  cycles_balance(): Promise<bigint>;
  get_economics_stats(): Promise<{
    balance: bigint;
    operational_mode: string;
    can_execute_outcall: boolean;
    estimated_cycles_per_day: bigint;
  }>;
  operational_mode(): Promise<string>;
  set_default_conversation(conversation_id: string): Promise<{ Ok: null } | { Err: string }>;
  clear_history(conversation_id: string): Promise<{ Ok: null } | { Err: string }>;
  add_sop(id: string, cron_expr: string, prompt: string, description: string): Promise<{ Ok: null } | { Err: string }>;
  remove_sop(id: string): Promise<{ Ok: null } | { Err: string }>;
  list_sops(): Promise<SopEntry[]>;
  sop_count(): Promise<bigint>;
  set_sop_enabled(id: string, enabled: boolean): Promise<{ Ok: null } | { Err: string }>;
  recall(key: string): Promise<ToolResult>;
  store(key: string, value: string): Promise<ToolResult>;
  forget(key: string): Promise<ToolResult>;
  list_memory_keys(): Promise<string[]>;
  schedule_task(id: string, cron_expr: string, payload: string): Promise<ToolResult>;
  cancel_task(id: string): Promise<ToolResult>;
  list_tasks(): Promise<ScheduledTask[]>;
  send_notification(message: string): Promise<ToolResult>;
  set_owner_principal(principal: Principal): Promise<{ Ok: null } | { Err: string }>;
  whoami(): Promise<Principal>;
  get_transform_func_name(): Promise<string>;
  handle_notification(payload: Uint8Array): Promise<{ Ok: null } | { Err: string }>;
}

function createActor(agent: HttpAgent): ActorSubclass<AgentService> {
  const encode = (obj: unknown): Uint8Array => {
    const json = JSON.stringify(obj);
    return new TextEncoder().encode(json);
  };

  const decode = (bytes: Uint8Array, type: string): unknown => {
    const json = new TextDecoder().decode(bytes);
    if (type === 'string') return json.replace(/^"(.*)"$/, '$1');
    return JSON.parse(json);
  };

  const call = async (method: string, arg?: unknown): Promise<unknown> => {
    const argBytes = arg ? encode(arg) : new Uint8Array();
    const result = await agent.call(AGENT_CANISTER_ID, method, {
      arg: argBytes,
    });
    if (result instanceof Uint8Array || ArrayBuffer.isView(result)) {
      return decode(new Uint8Array(result), 'json');
    }
    return result;
  };

  const service = {
    async chat(message: string) {
      return call('chat', { message }) as Promise<{ Ok: string } | { Err: string }>;
    },
    async chat_in_conversation(conversation_id: string, message: string) {
      return call('chat_in_conversation', { conversation_id, message }) as Promise<{ Ok: string } | { Err: string }>;
    },
    async get_history(limit: number) {
      return call('get_history', { limit }) as Promise<Message[]>;
    },
    async get_conversation(conversation_id: string) {
      return call('get_conversation', { conversation_id }) as Promise<Conversation | null>;
    },
    async list_conversations() {
      return call('list_conversations', {}) as Promise<string[]>;
    },
    async conversation_count() {
      return call('conversation_count', {}) as Promise<number>;
    },
    async cycles_balance() {
      return call('cycles_balance', {}) as Promise<bigint>;
    },
    async get_economics_stats() {
      return call('get_economics_stats', {}) as Promise<{
        balance: bigint;
        operational_mode: string;
        can_execute_outcall: boolean;
        estimated_cycles_per_day: bigint;
      }>;
    },
    async operational_mode() {
      return call('operational_mode', {}) as Promise<string>;
    },
    async set_default_conversation(conversation_id: string) {
      return call('set_default_conversation', { conversation_id }) as Promise<{ Ok: null } | { Err: string }>;
    },
    async clear_history(conversation_id: string) {
      return call('clear_history', { conversation_id }) as Promise<{ Ok: null } | { Err: string }>;
    },
    async add_sop(id: string, cron_expr: string, prompt: string, description: string) {
      return call('add_sop', { id, cron_expr, prompt, description }) as Promise<{ Ok: null } | { Err: string }>;
    },
    async remove_sop(id: string) {
      return call('remove_sop', { id }) as Promise<{ Ok: null } | { Err: string }>;
    },
    async list_sops() {
      return call('list_sops', {}) as Promise<SopEntry[]>;
    },
    async sop_count() {
      return call('sop_count', {}) as Promise<bigint>;
    },
    async set_sop_enabled(id: string, enabled: boolean) {
      return call('set_sop_enabled', { id, enabled }) as Promise<{ Ok: null } | { Err: string }>;
    },
    async recall(key: string) {
      return call('recall', { key }) as Promise<ToolResult>;
    },
    async store(key: string, value: string) {
      return call('store', { key, value }) as Promise<ToolResult>;
    },
    async forget(key: string) {
      return call('forget', { key }) as Promise<ToolResult>;
    },
    async list_memory_keys() {
      return call('list_memory_keys', {}) as Promise<string[]>;
    },
    async schedule_task(id: string, cron_expr: string, payload: string) {
      return call('schedule_task', { id, cron_expr, payload }) as Promise<ToolResult>;
    },
    async cancel_task(id: string) {
      return call('cancel_task', { id }) as Promise<ToolResult>;
    },
    async list_tasks() {
      return call('list_tasks', {}) as Promise<ScheduledTask[]>;
    },
    async send_notification(message: string) {
      return call('send_notification', { message }) as Promise<ToolResult>;
    },
    async set_owner_principal(principal: Principal) {
      return call('set_owner_principal', { principal }) as Promise<{ Ok: null } | { Err: string }>;
    },
    async whoami() {
      return call('whoami', {}) as Promise<Principal>;
    },
    async get_transform_func_name() {
      return call('get_transform_func_name', {}) as Promise<string>;
    },
    async handle_notification(payload: Uint8Array) {
      return call('handle_notification', { payload: Array.from(payload) }) as Promise<{ Ok: null } | { Err: string }>;
    },
  } as ActorSubclass<AgentService>;

  return service;
}

export function useAgent(agent: HttpAgent | null) {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const actor: ActorSubclass<AgentService> | null = agent ? createActor(agent) : null;

  const chat = useCallback(async (message: string): Promise<string> => {
    if (!actor) throw new Error('Not authenticated');
    setIsLoading(true);
    setError(null);
    try {
      const result = await actor.chat(message);
      if ('Ok' in result) return result.Ok;
      throw new Error(result.Err);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, [actor]);

  const getHistory = useCallback(async (limit: number = 50): Promise<Message[]> => {
    if (!actor) throw new Error('Not authenticated');
    return actor.get_history(limit);
  }, [actor]);

  const getConversations = useCallback(async (): Promise<string[]> => {
    if (!actor) throw new Error('Not authenticated');
    return actor.list_conversations();
  }, [actor]);

  const getConversation = useCallback(async (id: string): Promise<Conversation | null> => {
    if (!actor) throw new Error('Not authenticated');
    return actor.get_conversation(id);
  }, [actor]);

  const cyclesBalance = useCallback(async (): Promise<bigint> => {
    if (!actor) throw new Error('Not authenticated');
    return actor.cycles_balance();
  }, [actor]);

  const getEconomicsStats = useCallback(async (): Promise<EconomicsStats> => {
    if (!actor) throw new Error('Not authenticated');
    const stats = await actor.get_economics_stats();
    return {
      balance: Number(stats.balance),
      operational_mode: stats.operational_mode,
      can_execute_outcall: stats.can_execute_outcall,
      estimated_cycles_per_day: Number(stats.estimated_cycles_per_day),
    };
  }, [actor]);

  const getOperationalMode = useCallback(async (): Promise<string> => {
    if (!actor) throw new Error('Not authenticated');
    return actor.operational_mode();
  }, [actor]);

  const listSops = useCallback(async (): Promise<SopEntry[]> => {
    if (!actor) throw new Error('Not authenticated');
    return actor.list_sops();
  }, [actor]);

  const addSop = useCallback(async (
    id: string,
    cronExpr: string,
    prompt: string,
    description: string
  ): Promise<void> => {
    if (!actor) throw new Error('Not authenticated');
    const result = await actor.add_sop(id, cronExpr, prompt, description);
    if ('Err' in result) throw new Error(result.Err);
  }, [actor]);

  const removeSop = useCallback(async (id: string): Promise<void> => {
    if (!actor) throw new Error('Not authenticated');
    const result = await actor.remove_sop(id);
    if ('Err' in result) throw new Error(result.Err);
  }, [actor]);

  const recall = useCallback(async (key: string): Promise<ToolResult> => {
    if (!actor) throw new Error('Not authenticated');
    return actor.recall(key);
  }, [actor]);

  const store = useCallback(async (key: string, value: string): Promise<ToolResult> => {
    if (!actor) throw new Error('Not authenticated');
    return actor.store(key, value);
  }, [actor]);

  const listTasks = useCallback(async (): Promise<ScheduledTask[]> => {
    if (!actor) throw new Error('Not authenticated');
    return actor.list_tasks();
  }, [actor]);

  const whoami = useCallback(async (): Promise<string> => {
    if (!actor) throw new Error('Not authenticated');
    return actor.whoami().toText();
  }, [actor]);

  return {
    isLoading,
    error,
    chat,
    getHistory,
    getConversations,
    getConversation,
    cyclesBalance,
    getEconomicsStats,
    getOperationalMode,
    listSops,
    addSop,
    removeSop,
    recall,
    store,
    listTasks,
    whoami,
  };
}