import React, { useState } from 'react';
import { useWorkspaceStore } from '../../lib/state/workspaceStore';
import { invoke } from '@tauri-apps/api/core';
import '../../styles/panels/panels.css';

const DistillPanel: React.FC = () => {
    const { selectedForDistill, setSelectedForDistill, viewMode } = useWorkspaceStore();
    const [isDistilling, setIsDistilling] = useState(false);
    const [distillResult, setDistillResult] = useState<string | null>(null);

    // Only show the panel if viewMode is 'distill'
    if (viewMode !== 'distill') {
        return null;
    }

    const unselectMonad = (id: string) => {
        setSelectedForDistill(selectedForDistill.filter(m => m.id !== id));
    };

    const clearSelection = () => {
        setSelectedForDistill([]);
        setDistillResult(null);
    };

    const handleDistill = async () => {
        setIsDistilling(true);
        try {
            const monad_ids = selectedForDistill.map(m => m.id);
            // Wait for backend distillation logic if we're in Tauri
            if (typeof window !== 'undefined' && '__TAURI__' in window) {
                const source = await invoke<string>('distill_from_selection', { monads: monad_ids });
                setDistillResult(source);
            } else {
                setDistillResult('// Distilled source \n// (Mock result as Tauri is inactive)\n// Selected nodes: ' + monad_ids.length);
            }
        } catch (error) {
            console.error('Distill failed', error);
            setDistillResult('// Error distilling source: \n// ' + String(error));
        } finally {
            setIsDistilling(false);
        }
    };

    return (
        <div className="inspector" style={{ left: '2rem', right: 'auto', width: '380px' }}>
            <header style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <h2>Fuentes / Distill</h2>
                <button 
                    onClick={clearSelection} 
                    style={{ background: 'transparent', border: 'none', color: 'var(--text-dim)', cursor: 'pointer', fontFamily: 'monospace', fontSize: '1.25rem', padding: '0 0.5rem' }}
                >×</button>
            </header>
            
            <p style={{ fontSize: '0.8rem', color: 'var(--text-dim)', marginBottom: '1rem' }}>
                {selectedForDistill.length} Mónada(s) capturada(s) en Lazo.
            </p>

            <div style={{ maxHeight: '200px', overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '0.5rem', marginBottom: '1rem', paddingRight: '0.5rem' }}>
                {selectedForDistill.map(monad => (
                    <div key={monad.id} className="history-item" style={{ gridTemplateColumns: '1fr auto', alignItems: 'center' }}>
                        <div>
                            <span className="action">{monad.name}</span>
                            <div className="time">{monad.ring ? `Anillo ${monad.ring}` : 'Anillo ?'} — {monad.kind}</div>
                        </div>
                        <button 
                            onClick={() => unselectMonad(monad.id)}
                            style={{ background: 'rgba(255, 62, 131, 0.1)', border: '1px solid rgba(255, 62, 131, 0.3)', color: 'var(--tertiary)', borderRadius: '4px', cursor: 'pointer', padding: '0.2rem 0.5rem' }}
                        >X</button>
                    </div>
                ))}
            </div>

            {distillResult ? (
                <div style={{ marginTop: '1rem', borderTop: '1px solid var(--glass-border)', paddingTop: '1rem' }}>
                    <h3 style={{ fontSize: '0.8rem', color: 'var(--accent-primary)', marginBottom: '0.5rem', letterSpacing: '1px' }}>SOURCE GENERADO</h3>
                    <div className="code-viewer">
                        <pre><code>{distillResult}</code></pre>
                    </div>
                </div>
            ) : (
                <button 
                    className="btn-expand" 
                    style={{ width: '100%', marginTop: '0.5rem' }}
                    onClick={handleDistill}
                    disabled={isDistilling}
                >
                    {isDistilling ? 'CRISTALIZANDO...' : 'DISTILL SOURCE'}
                </button>
            )}
        </div>
    );
};

export default DistillPanel;
