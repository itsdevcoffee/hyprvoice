import { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { Save, RotateCcw, Check } from 'lucide-react';
import { invoke } from '../lib/ipc';
import { cn } from '../lib/utils';
import { useAppStore } from '../stores/appStore';
import RestartBanner from './RestartBanner';
import PathInput from './PathInput';

interface Config {
  model: {
    path: string;
    model_id: string;
    draft_model_path: string | null;
    language: string;
    prompt: string | null;
  };
  audio: {
    sample_rate: number;
    timeout_secs: number;
    save_audio_clips: boolean;
    audio_clips_path: string;
  };
  output: {
    append_space: boolean;
    refresh_command: string | null;
  };
}

export default function Settings() {
  const { isRecording } = useAppStore();
  const [config, setConfig] = useState<Config | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [needsRestart, setNeedsRestart] = useState(false);
  const [restarting, setRestarting] = useState(false);
  const [restartSuccess, setRestartSuccess] = useState(false);
  const [restartError, setRestartError] = useState<string | null>(null);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      setLoading(true);
      const cfg = await invoke<Config>('get_config');
      setConfig(cfg);
    } catch (error) {
      console.error('Failed to load config:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    if (!config) return;

    try {
      setSaving(true);
      await invoke('save_config', { config });
      setSaved(true);
      setNeedsRestart(true); // Show restart banner
      setRestartSuccess(false);
      setRestartError(null);
      setTimeout(() => setSaved(false), 3000);
    } catch (error) {
      console.error('Failed to save config:', error);
      alert(`Failed to save: ${error}`);
    } finally {
      setSaving(false);
    }
  };

  const handleRestart = async () => {
    if (isRecording) {
      setRestartError('Cannot restart while recording. Stop first.');
      return;
    }

    try {
      setRestarting(true);
      setRestartError(null);
      await invoke('restart_daemon');
      setRestartSuccess(true);
      setNeedsRestart(false);

      // Auto-dismiss success message after 3 seconds
      setTimeout(() => {
        setRestartSuccess(false);
      }, 3000);
    } catch (error) {
      console.error('Failed to restart daemon:', error);
      setRestartError(String(error));
    } finally {
      setRestarting(false);
    }
  };

  const handleDismissBanner = () => {
    setNeedsRestart(false);
    setRestartSuccess(false);
    setRestartError(null);
  };

  const handleReset = () => {
    if (confirm('Reset all settings to loaded values?')) {
      loadConfig();
    }
  };

  if (loading || !config) {
    return (
      <div className="glass-panel p-12 text-center">
        <div className="text-gray-400">Loading settings...</div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Restart Banner */}
      <RestartBanner
        show={needsRestart || restartSuccess || restartError !== null}
        isRecording={isRecording}
        restarting={restarting}
        success={restartSuccess}
        error={restartError}
        onRestart={handleRestart}
        onDismiss={handleDismissBanner}
      />

      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-white mb-2">Settings</h1>
          <p className="text-gray-400">Configure hyprvoice behavior</p>
        </div>

        {/* Action buttons */}
        <div className="flex gap-3">
          <motion.button
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
            onClick={handleReset}
            disabled={saving}
            className="glass-button px-4 py-2 text-sm flex items-center gap-2 text-gray-300"
          >
            <RotateCcw className="w-4 h-4" />
            Reset
          </motion.button>

          <motion.button
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
            onClick={handleSave}
            disabled={saving}
            className={cn(
              'px-6 py-2 text-sm flex items-center gap-2 rounded-lg transition-all duration-200',
              'backdrop-blur-md border-2 font-medium',
              saved
                ? 'bg-green-500/20 border-green-500 text-green-400'
                : 'bg-cyan-500/20 border-cyan-500 text-cyan-400 hover:bg-cyan-500/30'
            )}
          >
            {saved ? (
              <>
                <Check className="w-4 h-4" />
                Saved!
              </>
            ) : saving ? (
              <>
                <motion.div animate={{ rotate: 360 }} transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}>
                  <RotateCcw className="w-4 h-4" />
                </motion.div>
                Saving...
              </>
            ) : (
              <>
                <Save className="w-4 h-4" />
                Save Changes
              </>
            )}
          </motion.button>
        </div>
      </div>

      {/* Settings Sections */}
      <div className="space-y-6">
        {/* Model Settings */}
        <SettingsSection title="Model Configuration">
          <SettingRow label="Model ID" description="Whisper model identifier">
            <select
              value={config.model.model_id}
              onChange={(e) => setConfig({ ...config, model: { ...config.model, model_id: e.target.value }})}
              className="glass-input"
            >
              <option value="openai/whisper-tiny">Tiny (fast, less accurate)</option>
              <option value="openai/whisper-tiny.en">Tiny English-only</option>
              <option value="openai/whisper-base">Base (balanced)</option>
              <option value="openai/whisper-base.en">Base English-only</option>
              <option value="openai/whisper-small">Small (more accurate)</option>
              <option value="openai/whisper-small.en">Small English-only</option>
              <option value="openai/whisper-large-v3-turbo">Large V3 Turbo (recommended)</option>
            </select>
          </SettingRow>

          <SettingRow label="Language" description="Transcription language code">
            <input
              type="text"
              value={config.model.language}
              onChange={(e) => setConfig({ ...config, model: { ...config.model, language: e.target.value }})}
              className="glass-input"
              placeholder="en"
            />
          </SettingRow>

          <SettingRow label="Model Path" description="Path to model file or directory (safetensors or GGUF)">
            <PathInput
              value={config.model.path}
              onChange={(newPath) => setConfig({ ...config, model: { ...config.model, path: newPath }})}
              type="any"
              placeholder="~/.local/share/hyprvoice/models/..."
              label="Browse for model"
            />
          </SettingRow>

          <SettingRow label="Technical Vocabulary" description="Custom prompts to bias transcription (optional)">
            <textarea
              value={config.model.prompt || ''}
              onChange={(e) => setConfig({ ...config, model: { ...config.model, prompt: e.target.value || null }})}
              className="glass-input font-mono text-xs h-20"
              placeholder="async, await, kubernetes, docker, typescript..."
            />
          </SettingRow>
        </SettingsSection>

        {/* Audio Settings */}
        <SettingsSection title="Audio Configuration">
          <SettingRow label="Recording Timeout" description="Maximum recording duration in seconds">
            <div className="flex items-center gap-3">
              <input
                type="range"
                min="10"
                max="300"
                value={config.audio.timeout_secs}
                onChange={(e) => setConfig({ ...config, audio: { ...config.audio, timeout_secs: parseInt(e.target.value) }})}
                className="flex-1"
              />
              <span className="text-white font-mono text-sm w-12">{config.audio.timeout_secs}s</span>
            </div>
          </SettingRow>

          <SettingRow label="Save Audio Clips" description="Save recordings to disk for debugging">
            <Toggle
              checked={config.audio.save_audio_clips}
              onChange={(checked) => setConfig({ ...config, audio: { ...config.audio, save_audio_clips: checked }})}
            />
          </SettingRow>

          {config.audio.save_audio_clips && (
            <SettingRow label="Audio Clips Path" description="Directory to save WAV files">
              <PathInput
                value={config.audio.audio_clips_path}
                onChange={(newPath) => setConfig({ ...config, audio: { ...config.audio, audio_clips_path: newPath }})}
                type="directory"
                placeholder="~/.local/share/hyprvoice/recordings"
                label="Browse for folder"
              />
            </SettingRow>
          )}
        </SettingsSection>

        {/* Output Settings */}
        <SettingsSection title="Output Configuration">
          <SettingRow label="Append Space" description="Add space after transcribed text">
            <Toggle
              checked={config.output.append_space}
              onChange={(checked) => setConfig({ ...config, output: { ...config.output, append_space: checked }})}
            />
          </SettingRow>

          <SettingRow label="Status Bar Refresh Command" description="Command to update Waybar/Polybar (e.g., 'pkill -RTMIN+8 waybar')">
            <input
              type="text"
              value={config.output.refresh_command || ''}
              onChange={(e) => setConfig({ ...config, output: { ...config.output, refresh_command: e.target.value || null }})}
              className="glass-input font-mono text-xs"
              placeholder="pkill -RTMIN+8 waybar"
            />
          </SettingRow>
        </SettingsSection>
      </div>
    </div>
  );
}

interface SettingsSectionProps {
  title: string;
  children: React.ReactNode;
}

function SettingsSection({ title, children }: SettingsSectionProps) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      className="glass-panel p-6"
    >
      <h2 className="text-xl font-semibold text-white mb-6">{title}</h2>
      <div className="space-y-6">{children}</div>
    </motion.div>
  );
}

interface SettingRowProps {
  label: string;
  description: string;
  children: React.ReactNode;
}

function SettingRow({ label, description, children }: SettingRowProps) {
  return (
    <div className="flex items-start justify-between gap-6">
      <div className="flex-1">
        <label className="text-white font-medium text-sm block mb-1">{label}</label>
        <p className="text-gray-400 text-xs">{description}</p>
      </div>
      <div className="w-80">{children}</div>
    </div>
  );
}

interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
}

function Toggle({ checked, onChange }: ToggleProps) {
  return (
    <button
      onClick={() => onChange(!checked)}
      className={cn(
        'relative w-12 h-6 rounded-full transition-all duration-200',
        'backdrop-blur-md border',
        checked
          ? 'bg-cyan-500/30 border-cyan-500'
          : 'bg-white/10 border-white/20'
      )}
    >
      <motion.div
        animate={{ x: checked ? 24 : 2 }}
        transition={{ type: 'spring', stiffness: 500, damping: 30 }}
        className={cn(
          'absolute top-0.5 w-5 h-5 rounded-full',
          checked ? 'bg-cyan-500' : 'bg-gray-400'
        )}
      />
    </button>
  );
}
