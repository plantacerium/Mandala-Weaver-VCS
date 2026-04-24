import React, { useState, useRef, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface TerminalLine {
  type: 'input' | 'output' | 'error' | 'info';
  content: string;
  timestamp: Date;
}

interface CliResponse {
  success: boolean;
  output: string;
  error: string | null;
}

const AVAILABLE_COMMANDS = [
  { cmd: 'bindu', desc: 'Initialize new Mandala project' },
  { cmd: 'telemetry', desc: 'Show ecosystem statistics' },
  { cmd: 'status', desc: 'Display Mandala status' },
  { cmd: 'crystallize <file> [msg]', desc: 'Create new ring from file' },
  { cmd: 'distill [ring|vector]', desc: 'Extract source code' },
  { cmd: 'lineage [name]', desc: 'Show monad lineage' },
  { cmd: 'spectrum <name>', desc: 'Show semantic evolution' },
  { cmd: 'inspect <name> [--full]', desc: 'Inspect monad details' },
  { cmd: 'echo <ring> [name]', desc: 'Echo monads to new ring' },
  { cmd: 'vector <angle>', desc: 'Open vector space' },
  { cmd: 'focus <pattern>', desc: 'Focus monads by pattern' },
  { cmd: 'dormant', desc: 'Enter dormant state' },
  { cmd: 'synthesize <vec> [with]', desc: 'Synthesize from vectors' },
  { cmd: 'absorb [remote]', desc: 'Absorb from remote' },
  { cmd: 'emanate [remote]', desc: 'Emanate to remote' },
  { cmd: 'help', desc: 'Show this help' },
  { cmd: 'clear', desc: 'Clear terminal' },
];

const TerminalPanel: React.FC = () => {
  const [isOpen, setIsOpen] = useState(false);
  const [lines, setLines] = useState<TerminalLine[]>([
    { type: 'info', content: 'Mandala CLI v1.0.0 - Type "help" for available commands', timestamp: new Date() }
  ]);
  const [input, setInput] = useState('');
  const [isProcessing, setIsProcessing] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const terminalRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (terminalRef.current) {
      terminalRef.current.scrollTop = terminalRef.current.scrollHeight;
    }
  }, [lines]);

  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus();
    }
  }, [isOpen]);

  const isTauri = (): boolean => {
    return typeof window !== 'undefined' && ('__TAURI__' in window || '__TAURI_INTERNALS__' in window);
  };

  const parseCommand = (cmdStr: string): { command: string; args: string[] } => {
    const parts = cmdStr.trim().split(/\s+/);
    return {
      command: parts[0]?.toLowerCase() || '',
      args: parts.slice(1)
    };
  };

  const executeCommand = async (cmdStr: string): Promise<void> => {
    if (!cmdStr.trim()) return;

    const { command, args } = parseCommand(cmdStr);

    setLines(prev => [...prev, { type: 'input', content: `> ${cmdStr}`, timestamp: new Date() }]);

    if (command === 'help') {
      const helpText = AVAILABLE_COMMANDS.map(c => `  ${c.cmd.padEnd(24)} ${c.desc}`).join('\n');
      setLines(prev => [...prev, { type: 'output', content: helpText, timestamp: new Date() }]);
      return;
    }

    if (command === 'clear') {
      setLines([]);
      return;
    }

    if (!isTauri()) {
      setLines(prev => [...prev, { 
        type: 'error', 
        content: 'Tauri runtime not detected. Run in GUI mode to use CLI commands.', 
        timestamp: new Date() 
      }]);
      return;
    }

    setIsProcessing(true);

    try {
      let response: CliResponse | null = null;

      switch (command) {
        case 'bindu':
          response = await invoke<CliResponse>('cli_bindu');
          break;
        case 'telemetry':
          response = await invoke<CliResponse>('cli_telemetry', { verbose: args.includes('-v') || args.includes('--verbose') });
          break;
        case 'status':
          response = await invoke<CliResponse>('cli_status', { verbose: args.includes('-v') || args.includes('--verbose') });
          break;
        case 'crystallize':
          if (args.length === 0) {
            setLines(prev => [...prev, { type: 'error', content: 'Usage: crystallize <file> [message]', timestamp: new Date() }]);
            setIsProcessing(false);
            return;
          }
          const filePath = args[0];
          const message = args.slice(1).join(' ');
          response = await invoke<CliResponse>('cli_crystallize', { filePath, message });
          break;
        case 'distill':
          const targetRing = args[0] && !args[0].startsWith('-') ? parseInt(args[0]) : undefined;
          const vector = args.find(a => ['CORE', 'IO', 'UI', 'DATA'].includes(a));
          response = await invoke<CliResponse>('cli_distill', { targetRing, vector });
          break;
        case 'lineage':
          const monadName = args[0];
          const limit = args.find(a => a.startsWith('--limit='))?.split('=')[1];
          response = await invoke<CliResponse>('cli_lineage', { monadName, limit: limit ? parseInt(limit) : undefined });
          break;
        case 'spectrum':
          if (args.length === 0) {
            setLines(prev => [...prev, { type: 'error', content: 'Usage: spectrum <monad_name>', timestamp: new Date() }]);
            setIsProcessing(false);
            return;
          }
          response = await invoke<CliResponse>('cli_spectrum', { monadName: args[0] });
          break;
        case 'inspect':
          if (args.length === 0) {
            setLines(prev => [...prev, { type: 'error', content: 'Usage: inspect <monad_name> [--full]', timestamp: new Date() }]);
            setIsProcessing(false);
            return;
          }
          const inspectName = args[0];
          const full = args.includes('--full') || args.includes('-f');
          response = await invoke<CliResponse>('cli_inspect', { monadName: inspectName, full });
          break;
        case 'echo':
          if (args.length === 0) {
            setLines(prev => [...prev, { type: 'error', content: 'Usage: echo <ring_id> [monad_name]', timestamp: new Date() }]);
            setIsProcessing(false);
            return;
          }
          const echoRing = parseInt(args[0]);
          const echoName = args[1];
          response = await invoke<CliResponse>('cli_echo', { ringId: echoRing, monadName: echoName });
          break;
        case 'vector':
          if (args.length === 0) {
            setLines(prev => [...prev, { type: 'error', content: 'Usage: vector <angle|vector_name>', timestamp: new Date() }]);
            setIsProcessing(false);
            return;
          }
          const angleOrVec = args[0];
          const angle = parseFloat(angleOrVec) || 0;
          if (!isNaN(angle)) {
            response = await invoke<CliResponse>('cli_vector', { angle });
          }
          break;
        case 'focus':
          if (args.length === 0) {
            setLines(prev => [...prev, { type: 'error', content: 'Usage: focus <pattern>', timestamp: new Date() }]);
            setIsProcessing(false);
            return;
          }
          response = await invoke<CliResponse>('cli_focus', { monadPattern: args.join(' ') });
          break;
        case 'dormant':
          response = await invoke<CliResponse>('cli_dormant');
          break;
        case 'synthesize':
          if (args.length === 0) {
            setLines(prev => [...prev, { type: 'error', content: 'Usage: synthesize <vector> [with_vector]', timestamp: new Date() }]);
            setIsProcessing(false);
            return;
          }
          response = await invoke<CliResponse>('cli_synthesize', { vector: args[0], withVector: args[1] });
          break;
        case 'absorb':
          response = await invoke<CliResponse>('cli_absorb', { remote: args[0] });
          break;
        case 'emanate':
          response = await invoke<CliResponse>('cli_emanate', { remote: args[0] });
          break;
        default:
          setLines(prev => [...prev, { 
            type: 'error', 
            content: `Unknown command: ${command}. Type "help" for available commands.`, 
            timestamp: new Date() 
          }]);
          setIsProcessing(false);
          return;
      }

      if (response) {
        if (response.success) {
          setLines(prev => [...prev, { type: 'output', content: response.output, timestamp: new Date() }]);
        } else {
          setLines(prev => [...prev, { type: 'error', content: response.error || 'Unknown error', timestamp: new Date() }]);
        }
      }
    } catch (error) {
      setLines(prev => [...prev, { 
        type: 'error', 
        content: `Execution error: ${error}`, 
        timestamp: new Date() 
      }]);
    }

    setIsProcessing(false);
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && !isProcessing) {
      executeCommand(input);
      setInput('');
    }
  };

  const toggleTerminal = () => {
    setIsOpen(!isOpen);
  };

  return (
    <>
      <button 
        className="terminal-toggle"
        onClick={toggleTerminal}
        title={isOpen ? 'Close Terminal' : 'Open Terminal'}
        style={{
          position: 'fixed',
          bottom: isOpen ? '320px' : '20px',
          right: '20px',
          zIndex: 1000,
          padding: '10px 16px',
          background: isOpen ? '#e85d04' : '#1a1a2e',
          color: '#fff',
          border: '1px solid #e85d04',
          borderRadius: '6px',
          cursor: 'pointer',
          fontFamily: "'JetBrains Mono', monospace",
          fontSize: '12px',
          transition: 'all 0.3s ease',
          display: 'flex',
          alignItems: 'center',
          gap: '8px'
        }}
      >
        <span style={{ fontSize: '14px' }}>⌨</span>
        <span>{isOpen ? 'Close CLI' : 'CLI'}</span>
      </button>

      {isOpen && (
        <div 
          className="terminal-panel"
          ref={terminalRef}
          style={{
            position: 'fixed',
            bottom: 0,
            left: 0,
            right: 0,
            height: '300px',
            background: '#0d0d0d',
            borderTop: '2px solid #e85d04',
            fontFamily: "'JetBrains Mono', monospace",
            fontSize: '13px',
            display: 'flex',
            flexDirection: 'column',
            zIndex: 999
          }}
        >
          <div 
            style={{
              padding: '8px 16px',
              background: '#1a1a2e',
              borderBottom: '1px solid #333',
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center'
            }}
          >
            <span style={{ color: '#e85d04', fontWeight: 'bold' }}>MANDALA CLI</span>
            <span style={{ color: '#666', fontSize: '11px' }}>Type 'help' for commands</span>
          </div>

          <div 
            className="terminal-output"
            style={{
              flex: 1,
              overflow: 'auto',
              padding: '12px 16px',
              color: '#ccc'
            }}
          >
            {lines.map((line, idx) => (
              <div 
                key={idx}
                style={{
                  color: line.type === 'input' ? '#4ecdc4' : 
                         line.type === 'error' ? '#ff6b6b' : 
                         line.type === 'info' ? '#ffd93d' : '#ccc',
                  whiteSpace: 'pre-wrap',
                  marginBottom: '4px',
                  lineHeight: '1.5'
                }}
              >
                {line.content}
              </div>
            ))}
          </div>

          <div 
            style={{
              padding: '8px 16px',
              background: '#1a1a2e',
              borderTop: '1px solid #333',
              display: 'flex',
              alignItems: 'center',
              gap: '8px'
            }}
          >
            <span style={{ color: '#4ecdc4' }}>&gt;</span>
            <input
              ref={inputRef}
              type="text"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Enter command..."
              disabled={isProcessing}
              style={{
                flex: 1,
                background: 'transparent',
                border: 'none',
                color: '#fff',
                fontFamily: "'JetBrains Mono', monospace",
                fontSize: '13px',
                outline: 'none'
              }}
            />
            {isProcessing && <span style={{ color: '#ffd93d' }}>processing...</span>}
          </div>
        </div>
      )}
    </>
  );
};

export default TerminalPanel;