import { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { FolderOpen, FileText, Check, X, AlertCircle, Loader2 } from 'lucide-react';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '../lib/ipc';
import { cn } from '../lib/utils';

interface PathInputProps {
  value: string;
  onChange: (value: string) => void;
  type: 'file' | 'directory' | 'any';
  placeholder?: string;
  label?: string;
}

interface PathValidation {
  valid: boolean;
  exists: boolean;
  is_file: boolean;
  is_directory: boolean;
  expanded_path: string;
  message: string;
}

export default function PathInput({ value, onChange, type, placeholder, label }: PathInputProps) {
  const [validation, setValidation] = useState<PathValidation | null>(null);
  const [validating, setValidating] = useState(false);

  // Debounced validation
  useEffect(() => {
    if (!value) {
      setValidation(null);
      return;
    }

    setValidating(true);
    const timer = setTimeout(async () => {
      try {
        const result = await invoke<PathValidation>('validate_path', { path: value });
        setValidation(result);
      } catch (error) {
        console.error('Validation error:', error);
        setValidation(null);
      } finally {
        setValidating(false);
      }
    }, 500); // Validate 500ms after typing stops

    return () => clearTimeout(timer);
  }, [value]);

  const handleBrowse = async () => {
    try {
      const selected = await open({
        directory: type === 'directory',
        multiple: false,
        defaultPath: value,
      });
      if (selected) {
        onChange(selected as string);
      }
    } catch (error) {
      console.error('Failed to open picker:', error);
    }
  };

  // Determine validation icon
  const ValidationIcon = () => {
    if (validating) {
      return (
        <motion.div animate={{ rotate: 360 }} transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}>
          <Loader2 className="w-4 h-4 text-gray-400" />
        </motion.div>
      );
    }

    if (!validation) return null;

    const { exists, is_file, is_directory } = validation;

    // Check type match
    const typeMatches =
      type === 'any' ||
      (type === 'file' && is_file) ||
      (type === 'directory' && is_directory);

    if (exists && typeMatches) {
      return <Check className="w-4 h-4 text-green-400" />;
    } else if (exists && !typeMatches) {
      return (
        <div className="flex items-center gap-1">
          <AlertCircle className="w-4 h-4 text-yellow-400" />
        </div>
      );
    } else {
      return <X className="w-4 h-4 text-red-400" />;
    }
  };

  // Validation message
  const getValidationMessage = () => {
    if (!validation || validating) return null;

    const { exists, is_file, is_directory } = validation;
    const typeMatches =
      (type === 'file' && is_file) || (type === 'directory' && is_directory);

    if (exists && typeMatches) {
      return (
        <span className="text-green-400 text-xs">
          ✓ {type === 'any'
            ? is_file
              ? 'File exists'
              : 'Directory exists'
            : type === 'file'
            ? 'File exists'
            : 'Directory exists'}
        </span>
      );
    } else if (exists && !typeMatches) {
      return (
        <span className="text-yellow-400 text-xs">
          ⚠ Expected {type}, found {is_file ? 'file' : 'directory'}
        </span>
      );
    } else {
      return <span className="text-red-400 text-xs">✗ Path does not exist</span>;
    }
  };

  return (
    <div className="space-y-1">
      <div className="flex gap-2">
        <div className="relative flex-1">
          <input
            type="text"
            value={value}
            onChange={(e) => onChange(e.target.value)}
            className={cn(
              'glass-input font-mono text-xs pr-10',
              validation?.exists && (type === 'file' ? validation.is_file : validation.is_directory)
                ? 'border-green-500/30'
                : validation?.exists === false
                ? 'border-red-500/30'
                : ''
            )}
            placeholder={placeholder}
          />
          <div className="absolute right-3 top-1/2 -translate-y-1/2">
            <ValidationIcon />
          </div>
        </div>

        <motion.button
          whileHover={{ scale: 1.02 }}
          whileTap={{ scale: 0.98 }}
          onClick={handleBrowse}
          className="glass-button px-3 py-2 text-cyan-400 flex items-center gap-2"
          title={label || (type === 'file' ? 'Browse for file' : 'Browse for folder')}
        >
          {type === 'directory' ? (
            <FolderOpen className="w-4 h-4" />
          ) : (
            <FileText className="w-4 h-4" />
          )}
        </motion.button>
      </div>

      {/* Validation message */}
      {getValidationMessage()}
    </div>
  );
}
