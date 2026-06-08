import React from 'react';

export interface EconomicsStats {
  balance: number;
  operational_mode: string;
  can_execute_outcall: boolean;
  estimated_cycles_per_day: number;
}

export interface SopEntry {
  id: string;
  cron_expr: string;
  prompt: string;
  last_run: number;
  enabled: boolean;
  description: string;
}

interface DashboardProps {
  cyclesBalance: bigint;
  economicsStats: EconomicsStats | null;
  conversationCount: number;
  sopCount: number;
  sops: SopEntry[];
  operationalMode: string;
}

function formatBalance(balance: bigint): string {
  const n = Number(balance);
  if (n >= 1_000_000_000_000) return `${(n / 1_000_000_000_000).toFixed(2)}T`;
  if (n >= 1_000_000_000) return `${(n / 1_000_000_000).toFixed(2)}B`;
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`;
  return `${n}`;
}

function getModeColor(mode: string): string {
  switch (mode) {
    case 'Full': return 'bg-green-100 text-green-800';
    case 'Degraded': return 'bg-yellow-100 text-yellow-800';
    case 'Critical': return 'bg-red-100 text-red-800';
    default: return 'bg-gray-100 text-gray-800';
  }
}

export function Dashboard({
  cyclesBalance,
  economicsStats,
  conversationCount,
  sopCount,
  sops,
  operationalMode,
}: DashboardProps) {
  return (
    <div className="p-6 space-y-6">
      <h2 className="text-2xl font-bold text-gray-900">Dashboard</h2>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="bg-white rounded-lg shadow p-4">
          <p className="text-sm text-gray-500 mb-1">Cycles Balance</p>
          <p className="text-2xl font-bold text-gray-900">
            {formatBalance(cyclesBalance)}
            <span className="text-sm font-normal text-gray-500 ml-1">cycles</span>
          </p>
        </div>

        <div className="bg-white rounded-lg shadow p-4">
          <p className="text-sm text-gray-500 mb-1">Operational Mode</p>
          <span className={`inline-block px-2 py-1 rounded text-sm font-medium ${getModeColor(operationalMode)}`}>
            {operationalMode}
          </span>
          {economicsStats && !economicsStats.can_execute_outcall && (
            <p className="text-xs text-red-500 mt-1">Outcalls disabled — low balance</p>
          )}
        </div>

        <div className="bg-white rounded-lg shadow p-4">
          <p className="text-sm text-gray-500 mb-1">Daily Burn Rate</p>
          <p className="text-2xl font-bold text-gray-900">
            {formatBalance(BigInt(economicsStats?.estimated_cycles_per_day || 0))}
            <span className="text-sm font-normal text-gray-500 ml-1">cycles/day</span>
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="bg-white rounded-lg shadow p-4">
          <p className="text-sm text-gray-500 mb-1">Conversations</p>
          <p className="text-2xl font-bold text-gray-900">{conversationCount}</p>
        </div>

        <div className="bg-white rounded-lg shadow p-4">
          <p className="text-sm text-gray-500 mb-1">Active SOPs</p>
          <p className="text-2xl font-bold text-gray-900">{sopCount}</p>
        </div>

        <div className="bg-white rounded-lg shadow p-4">
          <p className="text-sm text-gray-500 mb-1">Estimated Days Left</p>
          {economicsStats && economicsStats.estimated_cycles_per_day > 0 ? (
            <p className="text-2xl font-bold text-gray-900">
              {Math.floor(Number(cyclesBalance) / Number(economicsStats.estimated_cycles_per_day))}
            </p>
          ) : (
            <p className="text-2xl font-bold text-gray-900">—</p>
          )}
        </div>
      </div>

      {sops.length > 0 && (
        <div className="bg-white rounded-lg shadow p-4">
          <h3 className="text-lg font-semibold text-gray-900 mb-3">Scheduled SOPs</h3>
          <div className="space-y-2">
            {sops.map(sop => (
              <div key={sop.id} className="flex items-center justify-between border-b pb-2">
                <div>
                  <p className="font-medium text-gray-900">{sop.id}</p>
                  <p className="text-sm text-gray-500">{sop.cron_expr} — {sop.description || 'No description'}</p>
                </div>
                <span className={`px-2 py-1 rounded text-xs font-medium ${
                  sop.enabled ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-500'
                }`}>
                  {sop.enabled ? 'Active' : 'Paused'}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      <div className="bg-blue-50 rounded-lg p-4">
        <h3 className="text-lg font-semibold text-blue-900 mb-2">Quick Reference</h3>
        <div className="grid grid-cols-2 gap-2 text-sm text-blue-800">
          <div>• HTTPS outcall cost: ~490M cycles</div>
          <div>• ic-llm call cost: ~10M cycles</div>
          <div>• Inter-canister call: ~10M cycles</div>
          <div>• 100 turns/day: ~150B cycles</div>
        </div>
      </div>
    </div>
  );
}