import { useState } from 'react';
import { invoke } from '../lib/ipc';
import { motion } from 'framer-motion';
import { Mic, Square, Zap, Cpu, Database } from 'lucide-react';
import { useAppStore } from '../stores/appStore';
import { cn } from '../lib/utils';

export default function Dashboard() {
  const { daemonStatus, isRecording, isProcessing, setRecording, setProcessing } = useAppStore();
  const [lastTranscription, setLastTranscription] = useState<string>('');

  const handleStartRecording = async () => {
    try {
      setRecording(true);
      await invoke('start_recording');
    } catch (error) {
      console.error('Failed to start recording:', error);
      setRecording(false);
    }
  };

  const handleStopRecording = async () => {
    try {
      setProcessing(true);
      const result = await invoke('stop_recording') as string;
      setLastTranscription(result);
      setRecording(false);
      setProcessing(false);
    } catch (error) {
      console.error('Failed to stop recording:', error);
      setRecording(false);
      setProcessing(false);
    }
  };

  return (
    <div className="p-8">
      {/* Header */}
      <motion.div
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
        className="mb-8"
      >
        <h1 className="text-4xl font-bold text-white mb-2">
          <span className="bg-clip-text text-transparent bg-gradient-to-r from-cyan-400 to-purple-500">
            hyprvoice
          </span>
        </h1>
        <p className="text-gray-400">GPU-Accelerated Voice Dictation</p>
      </motion.div>

      {/* Status Cards Grid */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
        <StatusCard
          icon={<Zap className="w-6 h-6" />}
          title="Daemon Status"
          value={daemonStatus.running ? 'Running' : 'Stopped'}
          status={daemonStatus.running ? 'active' : 'inactive'}
        />
        <StatusCard
          icon={<Cpu className="w-6 h-6" />}
          title="GPU"
          value={daemonStatus.gpuName || 'CPU Mode'}
          status={daemonStatus.gpuEnabled ? 'active' : 'inactive'}
        />
        <StatusCard
          icon={<Database className="w-6 h-6" />}
          title="Model"
          value={daemonStatus.modelLoaded ? 'Loaded' : 'Not Loaded'}
          status={daemonStatus.modelLoaded ? 'active' : 'inactive'}
        />
      </div>

      {/* Main Control Panel */}
      <motion.div
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ delay: 0.1 }}
        className="glass-panel p-8 mb-8"
      >
        <div className="text-center">
          <h2 className="text-2xl font-semibold text-white mb-6">
            {isRecording ? 'Recording...' : isProcessing ? 'Processing...' : 'Ready to Record'}
          </h2>

          {/* Record Button */}
          <motion.button
            whileHover={{ scale: 1.05 }}
            whileTap={{ scale: 0.95 }}
            onClick={isRecording ? handleStopRecording : handleStartRecording}
            disabled={isProcessing}
            className={cn(
              'relative w-24 h-24 rounded-full flex items-center justify-center',
              'backdrop-blur-md border-2 transition-all duration-300',
              isRecording
                ? 'bg-red-500/20 border-red-500 animate-pulse-glow'
                : 'bg-cyan-500/20 border-cyan-500 hover:bg-cyan-500/30',
              isProcessing && 'opacity-50 cursor-not-allowed'
            )}
          >
            {isRecording ? (
              <Square className="w-10 h-10 text-red-500" />
            ) : (
              <Mic className="w-10 h-10 text-cyan-500" />
            )}
          </motion.button>

          <p className="mt-4 text-gray-400 text-sm">
            {isRecording ? 'Click to stop' : 'Click to start recording'}
          </p>

          {/* Last Transcription */}
          {lastTranscription && (
            <motion.div
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              className="mt-8 glass-card p-4"
            >
              <p className="text-sm text-gray-400 mb-2">Last Transcription:</p>
              <p className="text-white">{lastTranscription}</p>
            </motion.div>
          )}
        </div>
      </motion.div>

      {/* Quick Stats */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <QuickStat label="Latency" value="<500ms" />
        <QuickStat label="Accuracy" value="98%" />
        <QuickStat label="Sessions" value="24" />
        <QuickStat label="Total Time" value="2.4h" />
      </div>
    </div>
  );
}

interface StatusCardProps {
  icon: React.ReactNode;
  title: string;
  value: string;
  status: 'active' | 'inactive';
}

function StatusCard({ icon, title, value, status }: StatusCardProps) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      whileHover={{ y: -4 }}
      className="glass-card p-6 cursor-default"
    >
      <div className="flex items-center gap-3 mb-3">
        <div className={cn(
          'p-2 rounded-lg',
          status === 'active' ? 'bg-cyan-500/20 text-cyan-500' : 'bg-gray-500/20 text-gray-500'
        )}>
          {icon}
        </div>
        <div className={cn(
          'w-2 h-2 rounded-full',
          status === 'active' ? 'bg-cyan-500 animate-pulse' : 'bg-gray-500'
        )} />
      </div>
      <p className="text-gray-400 text-sm mb-1">{title}</p>
      <p className="text-white text-lg font-semibold">{value}</p>
    </motion.div>
  );
}

interface QuickStatProps {
  label: string;
  value: string;
}

function QuickStat({ label, value }: QuickStatProps) {
  return (
    <div className="glass-card p-4 text-center">
      <p className="text-2xl font-bold text-white mb-1">{value}</p>
      <p className="text-gray-400 text-xs">{label}</p>
    </div>
  );
}
