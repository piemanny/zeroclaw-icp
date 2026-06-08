import React, { useState } from 'react';

interface SettingsProps {
  principal: string;
  onAddSop: (id: string, cronExpr: string, prompt: string, description: string) => Promise<void>;
  onRemoveSop: (id: string) => Promise<void>;
  onStore: (key: string, value: string) => Promise<void>;
  onRecall: (key: string) => Promise<string>;
}

export function Settings({ principal, onAddSop, onRemoveSop, onStore, onRecall }: SettingsProps) {
  const [activeTab, setActiveTab] = useState<'sops' | 'memory' | 'identity'>('sops');

  const [sopForm, setSopForm] = useState({
    id: '',
    cronExpr: '*/5 * * * *',
    prompt: '',
    description: '',
  });
  const [sopError, setSopError] = useState('');
  const [sopSuccess, setSopSuccess] = useState('');

  const [memoryForm, setMemoryForm] = useState({ key: '', value: '' });
  const [memoryResult, setMemoryResult] = useState('');
  const [memoryError, setMemoryError] = useState('');

  const handleAddSop = async (e: React.FormEvent) => {
    e.preventDefault();
    setSopError('');
    setSopSuccess('');
    try {
      await onAddSop(sopForm.id, sopForm.cronExpr, sopForm.prompt, sopForm.description);
      setSopSuccess(`SOP "${sopForm.id}" added successfully`);
      setSopForm({ id: '', cronExpr: '*/5 * * * *', prompt: '', description: '' });
    } catch (err) {
      setSopError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleRecall = async (e: React.FormEvent) => {
    e.preventDefault();
    setMemoryError('');
    setMemoryResult('');
    try {
      const result = await onRecall(memoryForm.key);
      setMemoryResult(result);
    } catch (err) {
      setMemoryError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleStore = async (e: React.FormEvent) => {
    e.preventDefault();
    setMemoryError('');
    try {
      await onStore(memoryForm.key, memoryForm.value);
      setMemoryResult('Stored successfully');
      setMemoryForm({ key: '', value: '' });
    } catch (err) {
      setMemoryError(err instanceof Error ? err.message : String(err));
    }
  };

  return (
    <div className="p-6">
      <h2 className="text-2xl font-bold text-gray-900 mb-4">Settings</h2>

      <div className="mb-4">
        <p className="text-sm text-gray-500 mb-1">Your Principal</p>
        <code className="text-sm bg-gray-100 px-2 py-1 rounded">{principal}</code>
      </div>

      <div className="flex gap-2 mb-6 border-b">
        {(['sops', 'memory', 'identity'] as const).map(tab => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`px-4 py-2 font-medium capitalize ${
              activeTab === tab
                ? 'border-b-2 border-blue-600 text-blue-600'
                : 'text-gray-500 hover:text-gray-700'
            }`}
          >
            {tab}
          </button>
        ))}
      </div>

      {activeTab === 'sops' && (
        <form onSubmit={handleAddSop} className="space-y-4">
          <h3 className="text-lg font-semibold">Add Scheduled SOP</h3>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">SOP ID</label>
            <input
              type="text"
              value={sopForm.id}
              onChange={e => setSopForm(f => ({ ...f, id: e.target.value }))}
              className="w-full rounded border px-3 py-2"
              placeholder="e.g. daily-briefing"
              required
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Cron Expression</label>
            <input
              type="text"
              value={sopForm.cronExpr}
              onChange={e => setSopForm(f => ({ ...f, cronExpr: e.target.value }))}
              className="w-full rounded border px-3 py-2"
              placeholder="*/5 * * * *"
            />
            <p className="text-xs text-gray-500 mt-1">Format: min hour day month dow</p>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Prompt</label>
            <textarea
              value={sopForm.prompt}
              onChange={e => setSopForm(f => ({ ...f, prompt: e.target.value }))}
              className="w-full rounded border px-3 py-2 h-24"
              placeholder="What should this SOP do?"
              required
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Description</label>
            <input
              type="text"
              value={sopForm.description}
              onChange={e => setSopForm(f => ({ ...f, description: e.target.value }))}
              className="w-full rounded border px-3 py-2"
              placeholder="Optional description"
            />
          </div>

          {sopError && <p className="text-red-600 text-sm">{sopError}</p>}
          {sopSuccess && <p className="text-green-600 text-sm">{sopSuccess}</p>}

          <button
            type="submit"
            className="bg-blue-600 text-white rounded px-4 py-2 hover:bg-blue-700"
          >
            Add SOP
          </button>
        </form>
      )}

      {activeTab === 'memory' && (
        <div className="space-y-6">
          <form onSubmit={handleRecall} className="space-y-4">
            <h3 className="text-lg font-semibold">Recall Value</h3>
            <div className="flex gap-2">
              <input
                type="text"
                value={memoryForm.key}
                onChange={e => setMemoryForm(f => ({ ...f, key: e.target.value }))}
                className="flex-1 rounded border px-3 py-2"
                placeholder="Key name"
                required
              />
              <button type="submit" className="bg-gray-800 text-white rounded px-4 py-2">
                Recall
              </button>
            </div>
          </form>

          {memoryResult && (
            <div className="bg-green-50 rounded p-3">
              <p className="text-sm font-medium text-green-800 mb-1">Result:</p>
              <pre className="text-sm text-green-700 whitespace-pre-wrap">{memoryResult}</pre>
            </div>
          )}

          <form onSubmit={handleStore} className="space-y-4">
            <h3 className="text-lg font-semibold">Store Value</h3>
            <input
              type="text"
              value={memoryForm.key}
              onChange={e => setMemoryForm(f => ({ ...f, key: e.target.value }))}
              className="w-full rounded border px-3 py-2"
              placeholder="Key name"
              required
            />
            <textarea
              value={memoryForm.value}
              onChange={e => setMemoryForm(f => ({ ...f, value: e.target.value }))}
              className="w-full rounded border px-3 py-2 h-24"
              placeholder="Value to store"
              required
            />
            {memoryError && <p className="text-red-600 text-sm">{memoryError}</p>}
            <button
              type="submit"
              className="bg-blue-600 text-white rounded px-4 py-2 hover:bg-blue-700"
            >
              Store
            </button>
          </form>
        </div>
      )}

      {activeTab === 'identity' && (
        <div className="space-y-4">
          <h3 className="text-lg font-semibold">Agent Identity Files</h3>
          <p className="text-sm text-gray-500">
            Identity files define your agent's personality, values, and operating constraints.
            These are stored in the agent's persistent memory.
          </p>
          <div className="bg-gray-50 rounded-lg p-4 space-y-3">
            <div>
              <p className="text-sm font-medium text-gray-700">IDENTITY.md</p>
              <p className="text-xs text-gray-500">Core personality and values</p>
            </div>
            <div>
              <p className="text-sm font-medium text-gray-700">USER.md</p>
              <p className="text-xs text-gray-500">User preferences and context</p>
            </div>
            <div>
              <p className="text-sm font-medium text-gray-700">SOUL.md</p>
              <p className="text-xs text-gray-500">Principles and operating constraints</p>
            </div>
          </div>
          <p className="text-xs text-gray-400">
            Use the memory store/recall functions above to edit identity files.
          </p>
        </div>
      )}
    </div>
  );
}