import { useState } from 'react';
import { motion } from 'framer-motion';
import { Terminal, Network, Download, Trash2, Copy } from 'lucide-react';
import { useAppStore } from '../stores/appStore';
import { cn } from '../lib/utils';

type DevTab = 'logs' | 'ipc' | 'diagnostics';

export default function DevTools() {
  const [activeTab, setActiveTab] = useState<DevTab>('logs');
  const { logs, ipcCalls, clearLogs, clearIPCCalls } = useAppStore();

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-white mb-2">Developer Tools</h1>
          <p className="text-gray-400">Debug, monitor, and diagnose hyprvoice</p>
        </div>
      </div>

      {/* Tabs */}
      <div className="glass-panel p-2">
        <div className="flex gap-2">
          <TabButton
            active={activeTab === 'logs'}
            onClick={() => setActiveTab('logs')}
            icon={<Terminal className="w-4 h-4" />}
            label="Live Logs"
          />
          <TabButton
            active={activeTab === 'ipc'}
            onClick={() => setActiveTab('ipc')}
            icon={<Network className="w-4 h-4" />}
            label="IPC Monitor"
          />
          <TabButton
            active={activeTab === 'diagnostics'}
            onClick={() => setActiveTab('diagnostics')}
            icon={<Download className="w-4 h-4" />}
            label="Diagnostics"
          />
        </div>
      </div>

      {/* Tab Content */}
      <motion.div
        key={activeTab}
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.2 }}
      >
        {activeTab === 'logs' && <LogsTab logs={logs} onClear={clearLogs} />}
        {activeTab === 'ipc' && <IPCTab calls={ipcCalls} onClear={clearIPCCalls} />}
        {activeTab === 'diagnostics' && <DiagnosticsTab />}
      </motion.div>
    </div>
  );
}

interface TabButtonProps {
  active: boolean;
  onClick: () => void;
  icon: React.ReactNode;
  label: string;
}

function TabButton({ active, onClick, icon, label }: TabButtonProps) {
  return (
    <motion.button
      whileHover={{ scale: 1.02 }}
      whileTap={{ scale: 0.98 }}
      onClick={onClick}
      className={cn(
        'flex items-center gap-2 px-4 py-2 rounded-lg transition-all duration-200',
        'backdrop-blur-sm border',
        active
          ? 'bg-cyan-500/20 border-cyan-500/50 text-cyan-400'
          : 'bg-white/5 border-white/10 text-gray-400 hover:bg-white/10 hover:text-white'
      )}
    >
      {icon}
      <span className="text-sm font-medium">{label}</span>
    </motion.button>
  );
}

interface LogsTabProps {
  logs: any[];
  onClear: () => void;
}

function LogsTab({ logs, onClear }: LogsTabProps) {
  return (
    <div className="glass-panel p-6">
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-semibold text-white">Live Daemon Logs</h2>
        <div className="flex gap-2">
          <button
            onClick={onClear}
            className="glass-button px-3 py-1.5 text-sm flex items-center gap-2 text-gray-300"
          >
            <Trash2 className="w-4 h-4" />
            Clear
          </button>
        </div>
      </div>

      {/* Log viewer */}
      <div className="bg-black/40 rounded-lg p-4 font-mono text-sm h-[500px] overflow-y-auto">
        {logs.length === 0 ? (
          <div className="text-gray-500 text-center py-8">
            No logs yet. Start recording to see activity.
          </div>
        ) : (
          <div className="space-y-1">
            {logs.map((log) => (
              <LogLine key={log.id} log={log} />
            ))}
          </div>
        )}
      </div>

      <p className="text-gray-500 text-xs mt-2">
        Showing last {logs.length} entries (max 500)
      </p>
    </div>
  );
}

interface LogLineProps {
  log: any;
}

function LogLine({ log }: LogLineProps) {
  const levelColors = {
    debug: 'text-gray-400',
    info: 'text-cyan-400',
    warn: 'text-yellow-400',
    error: 'text-red-400',
  };

  const timestamp = new Date(log.timestamp).toLocaleTimeString();

  // Check if log is new (within last 3 seconds)
  const isNew = Date.now() - log.timestamp < 3000;

  return (
    <motion.div
      initial={{ backgroundColor: 'rgba(0, 212, 255, 0.2)', x: -10 }}
      animate={{
        backgroundColor: isNew ? 'rgba(0, 212, 255, 0.1)' : 'rgba(0, 0, 0, 0)',
        x: 0
      }}
      transition={{ duration: 0.5, backgroundColor: { duration: 2 } }}
      className="flex gap-2 hover:bg-white/5 px-2 py-1 rounded border-l-2 border-transparent"
      style={{
        borderLeftColor: isNew ? 'rgba(0, 212, 255, 0.6)' : 'transparent',
      }}
    >
      <span className="text-gray-500">{timestamp}</span>
      <span className={cn('font-semibold uppercase text-xs', levelColors[log.level as keyof typeof levelColors])}>
        [{log.level}]
      </span>
      <span className="text-gray-300">{log.message}</span>
    </motion.div>
  );
}

interface IPCTabProps {
  calls: any[];
  onClear: () => void;
}

function IPCTab({ calls, onClear }: IPCTabProps) {
  return (
    <div className="glass-panel p-6">
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-semibold text-white">IPC Call Monitor</h2>
        <button
          onClick={onClear}
          className="glass-button px-3 py-1.5 text-sm flex items-center gap-2 text-gray-300"
        >
          <Trash2 className="w-4 h-4" />
          Clear
        </button>
      </div>

      {/* IPC call list */}
      <div className="space-y-2 max-h-[500px] overflow-y-auto">
        {calls.length === 0 ? (
          <div className="text-gray-500 text-center py-8 glass-card p-6">
            No IPC calls yet. Interact with the UI to see calls.
          </div>
        ) : (
          calls.map((call) => <IPCCallCard key={call.id} call={call} />)
        )}
      </div>

      <p className="text-gray-500 text-xs mt-4">
        Showing last {calls.length} calls (max 100)
      </p>
    </div>
  );
}

interface IPCCallCardProps {
  call: any;
}

function IPCCallCard({ call }: IPCCallCardProps) {
  const [expanded, setExpanded] = useState(false);
  const timestamp = new Date(call.timestamp).toLocaleTimeString();

  // Check if call is new (within last 3 seconds)
  const isNew = Date.now() - call.timestamp < 3000;

  return (
    <motion.div
      initial={{ scale: 0.98, opacity: 0, y: -10 }}
      animate={{
        scale: 1,
        opacity: 1,
        y: 0,
        boxShadow: isNew
          ? '0 0 20px rgba(0, 212, 255, 0.4)'
          : '0 0 0px rgba(0, 212, 255, 0)'
      }}
      transition={{
        scale: { duration: 0.3 },
        opacity: { duration: 0.3 },
        boxShadow: { duration: 2, ease: 'easeOut' }
      }}
      className={cn(
        'glass-card p-4 border-2 transition-colors duration-2000',
        isNew ? 'border-cyan-500/50' : 'border-white/10'
      )}
    >
      <div
        className="flex items-center justify-between cursor-pointer"
        onClick={() => setExpanded(!expanded)}
      >
        <div className="flex items-center gap-3">
          <motion.div
            className={cn(
              'w-2 h-2 rounded-full',
              call.error ? 'bg-red-500' : 'bg-green-500'
            )}
            animate={isNew ? { scale: [1, 1.5, 1] } : {}}
            transition={{ duration: 0.6, repeat: isNew ? 2 : 0 }}
          />
          <span className="text-cyan-400 font-mono text-sm">{call.command}</span>
          <span className="text-gray-500 text-xs">{timestamp}</span>
          <span className="text-gray-600 text-xs">{call.durationMs}ms</span>
        </div>
        <span className="text-gray-500 text-xs">{expanded ? '▼' : '▶'}</span>
      </div>

      {expanded && (
        <div className="mt-3 space-y-2 text-xs font-mono">
          {call.args && (
            <div>
              <p className="text-gray-400 mb-1">Args:</p>
              <pre className="bg-black/40 p-2 rounded text-gray-300 overflow-x-auto">
                {JSON.stringify(call.args, null, 2)}
              </pre>
            </div>
          )}
          {call.result && (
            <div>
              <p className="text-gray-400 mb-1">Result:</p>
              <pre className="bg-black/40 p-2 rounded text-green-300 overflow-x-auto">
                {JSON.stringify(call.result, null, 2)}
              </pre>
            </div>
          )}
          {call.error && (
            <div>
              <p className="text-gray-400 mb-1">Error:</p>
              <pre className="bg-black/40 p-2 rounded text-red-300 overflow-x-auto">
                {call.error}
              </pre>
            </div>
          )}
        </div>
      )}
    </motion.div>
  );
}

function DiagnosticsTab() {
  return (
    <div className="glass-panel p-6">
      <h2 className="text-xl font-semibold text-white mb-4">System Diagnostics</h2>

      <div className="space-y-4">
        {/* System Info */}
        <div className="glass-card p-4">
          <h3 className="text-white font-medium mb-3">System Information</h3>
          <div className="space-y-2 text-sm">
            <InfoRow label="Platform" value="Linux (Fedora 42)" />
            <InfoRow label="CPU Cores" value="16" />
            <InfoRow label="Total RAM" value="32 GB" />
            <InfoRow label="GPU" value="NVIDIA RTX 4090" />
            <InfoRow label="VRAM" value="24 GB" />
          </div>
        </div>

        {/* Daemon Info */}
        <div className="glass-card p-4">
          <h3 className="text-white font-medium mb-3">Daemon Status</h3>
          <div className="space-y-2 text-sm">
            <InfoRow label="Socket Path" value="~/.local/state/hyprvoice/daemon.sock" />
            <InfoRow label="Model" value="whisper-large-v3-turbo" />
            <InfoRow label="Backend" value="Candle (GPU)" />
            <InfoRow label="Uptime" value="2h 34m" />
          </div>
        </div>

        {/* Actions */}
        <div className="flex gap-3">
          <button className="glass-button px-4 py-2 text-sm flex items-center gap-2 text-white flex-1">
            <Download className="w-4 h-4" />
            Export Diagnostics
          </button>
          <button className="glass-button px-4 py-2 text-sm flex items-center gap-2 text-white flex-1">
            <Copy className="w-4 h-4" />
            Copy System Info
          </button>
        </div>
      </div>
    </div>
  );
}

interface InfoRowProps {
  label: string;
  value: string;
}

function InfoRow({ label, value }: InfoRowProps) {
  return (
    <div className="flex justify-between items-center">
      <span className="text-gray-400">{label}</span>
      <span className="text-white font-mono">{value}</span>
    </div>
  );
}
