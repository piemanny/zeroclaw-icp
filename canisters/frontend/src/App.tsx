import React, { useState, useEffect, useCallback } from 'react';
import { useAuth } from './hooks/useAuth';
import { useAgent } from './hooks/useAgent';
import { Chat } from './components/Chat';
import { Dashboard } from './components/Dashboard';
import { Settings } from './components/Settings';
import type { Message, SopEntry } from './hooks/types';

type Tab = 'chat' | 'dashboard' | 'settings';

export default function App() {
  const auth = useAuth();
  const agent = useAgent(auth.agent);

  const [activeTab, setActiveTab] = useState<Tab>('chat');
  const [messages, setMessages] = useState<Message[]>([]);
  const [conversationCount, setConversationCount] = useState(0);
  const [principal, setPrincipal] = useState('');
  const [sops, setSops] = useState<SopEntry[]>([]);
  const [cyclesBalance, setCyclesBalance] = useState(BigInt(0));
  const [operationalMode, setOperationalMode] = useState('Unknown');
  const [economicsStats, setEconomicsStats] = useState(null);

  const refreshStats = useCallback(async () => {
    if (!auth.isAuthenticated) return;
    try {
      const [balance, mode, econStats, sopList] = await Promise.all([
        agent.cyclesBalance(),
        agent.getOperationalMode(),
        agent.getEconomicsStats().catch(() => null),
        agent.listSops().catch(() => []),
      ]);
      setCyclesBalance(balance);
      setOperationalMode(mode);
      setEconomicsStats(econStats);
      setSops(sopList);
    } catch (err) {
      console.error('Failed to refresh stats:', err);
    }
  }, [auth.isAuthenticated, auth.agent, agent]);

  useEffect(() => {
    if (auth.isAuthenticated) {
      setPrincipal(auth.principal?.toText() || '');
      refreshStats();
      const interval = setInterval(refreshStats, 30_000);
      return () => clearInterval(interval);
    }
  }, [auth.isAuthenticated, auth.principal, refreshStats]);

  const loadHistory = useCallback(async () => {
    if (!auth.isAuthenticated) return;
    try {
      const history = await agent.getHistory(50);
      setMessages(history as Message[]);
    } catch (err) {
      console.error('Failed to load history:', err);
    }
  }, [auth.isAuthenticated, agent]);

  useEffect(() => {
    if (auth.isAuthenticated) {
      loadHistory();
    }
  }, [auth.isAuthenticated, loadHistory]);

  const handleSend = async (message: string) => {
    if (!auth.isAuthenticated) return;

    const userMsg: Message = {
      role: 'user',
      content: message,
      timestamp: Date.now() * 1_000_000,
    };
    setMessages(prev => [...prev, userMsg]);

    try {
      const response = await agent.chat(message);
      const assistantMsg: Message = {
        role: 'assistant',
        content: response,
        timestamp: Date.now() * 1_000_000,
      };
      setMessages(prev => [...prev, assistantMsg]);
      await refreshStats();
    } catch (err) {
      const errorMsg: Message = {
        role: 'assistant',
        content: `Error: ${err instanceof Error ? err.message : String(err)}`,
        timestamp: Date.now() * 1_000_000,
      };
      setMessages(prev => [...prev, errorMsg]);
    }
  };

  const handleAddSop = async (id: string, cronExpr: string, prompt: string, description: string) => {
    await agent.addSop(id, cronExpr, prompt, description);
    await refreshStats();
  };

  const handleRemoveSop = async (id: string) => {
    await agent.removeSop(id);
    await refreshStats();
  };

  const handleRecall = async (key: string) => {
    const result = await agent.recall(key);
    return result.output;
  };

  const handleStore = async (key: string, value: string) => {
    await agent.store(key, value);
  };

  if (auth.isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50">
        <div className="text-center">
          <div className="text-4xl mb-4">🦀</div>
          <p className="text-gray-600">Initializing ZeroClaw...</p>
        </div>
      </div>
    );
  }

  if (!auth.isAuthenticated) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-purple-50">
        <div className="bg-white rounded-xl shadow-xl p-8 max-w-md w-full mx-4">
          <div className="text-center mb-6">
            <div className="text-5xl mb-4">🦀</div>
            <h1 className="text-2xl font-bold text-gray-900">ZeroClaw ICP</h1>
            <p className="text-gray-600 mt-2">
              Persistent autonomous AI agents on the Internet Computer
            </p>
          </div>
          {auth.error && (
            <div className="bg-red-50 text-red-700 p-3 rounded-lg mb-4 text-sm">
              {auth.error}
            </div>
          )}
          <button
            onClick={() => auth.login()}
            className="w-full bg-blue-600 text-white rounded-lg py-3 font-medium hover:bg-blue-700 transition"
          >
            Sign in with Internet Identity
          </button>
          <p className="text-xs text-gray-500 mt-4 text-center">
            Your agent will be cryptographically owned by you via Internet Identity.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50 flex flex-col">
      <header className="bg-white shadow-sm border-b">
        <div className="max-w-6xl mx-auto px-4 py-3 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <span className="text-2xl">🦀</span>
            <h1 className="text-xl font-bold text-gray-900">ZeroClaw ICP</h1>
          </div>
          <div className="flex items-center gap-4">
            <span className="text-sm text-gray-500">
              {auth.principal?.toText().slice(0, 8)}...
            </span>
            <button
              onClick={() => auth.logout()}
              className="text-sm text-gray-500 hover:text-gray-700"
            >
              Sign out
            </button>
          </div>
        </div>
      </header>

      <nav className="bg-white border-b">
        <div className="max-w-6xl mx-auto px-4">
          <div className="flex gap-1">
            {([
              { key: 'chat', label: 'Chat' },
              { key: 'dashboard', label: 'Dashboard' },
              { key: 'settings', label: 'Settings' },
            ] as const).map(tab => (
              <button
                key={tab.key}
                onClick={() => setActiveTab(tab.key)}
                className={`px-4 py-3 font-medium text-sm ${
                  activeTab === tab.key
                    ? 'border-b-2 border-blue-600 text-blue-600'
                    : 'text-gray-500 hover:text-gray-700'
                }`}
              >
                {tab.label}
              </button>
            ))}
          </div>
        </div>
      </nav>

      <main className="flex-1 max-w-6xl mx-auto w-full">
        {activeTab === 'chat' && (
          <Chat
            messages={messages}
            onSend={handleSend}
            isLoading={agent.isLoading}
          />
        )}
        {activeTab === 'dashboard' && (
          <Dashboard
            cyclesBalance={cyclesBalance}
            economicsStats={economicsStats}
            conversationCount={conversationCount}
            sopCount={sops.length}
            sops={sops}
            operationalMode={operationalMode}
          />
        )}
        {activeTab === 'settings' && (
          <Settings
            principal={auth.principal?.toText() || ''}
            onAddSop={handleAddSop}
            onRemoveSop={handleRemoveSop}
            onStore={handleStore}
            onRecall={handleRecall}
          />
        )}
      </main>
    </div>
  );
}