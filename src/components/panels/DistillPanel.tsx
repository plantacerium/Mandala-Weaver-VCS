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
            if (typeof window !== 'undefined' && '__TAURI__' in window) {
                const source = await invoke<string>('distill_from_selection', { monadIds: monad_ids });
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

    const handleExport = async () => {
        try {
            if (typeof window !== 'undefined' && '__TAURI__' in window) {
                await invoke('export_mandala_archive', { projectName: 'mandala-project', outputPath: '.' });
                alert('Mandala emanated to current directory (mandala.json)');
            }
        } catch (error) {
            console.error('Export failed', error);
        }
    };

    const handleImport = async () => {
        try {
            if (typeof window !== 'undefined' && '__TAURI__' in window) {
                await invoke('import_mandala_archive', { archivePath: './mandala.json' });
                window.location.reload();
            }
        } catch (error) {
            console.error('Import failed', error);
            alert('Import failed: ' + error);
        }
    };

    return (
        <div className="inspector" style={{ left: '2rem', right: 'auto', width: '380px' }}>
            <header style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <h2>Fuentes / Cooperación</h2>
                <button 
                    onClick={clearSelection} 
                    style={{ background: 'transparent', border: 'none', color: 'var(--text-dim)', cursor: 'pointer', fontFamily: 'monospace', fontSize: '1.25rem', padding: '0 0.5rem' }}
                >×</button>
            </header>
            
            <div style={{ display: 'flex', gap: '0.5rem', marginBottom: '1rem' }}>
                <button className="btn-expand" style={{ flex: 1, fontSize: '0.7rem', padding: '0.4rem' }} onClick={handleExport}>EMANATE (EXPORT)</button>
                <button className="btn-expand" style={{ flex: 1, fontSize: '0.7rem', padding: '0.4rem', background: 'rgba(0, 255, 128, 0.1)', borderColor: 'var(--accent-primary)' }} onClick={handleImport}>ABSORB (IMPORT)</button>
            </div>

            <p style={{ fontSize: '0.8rem', color: 'var(--text-dim)', marginBottom: '0.5rem' }}>
                {selectedForDistill.length} Mónada(s) capturada(s) en Lazo.
            </p>

            <div style={{ maxHeight: '150px', overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '0.5rem', marginBottom: '1rem', paddingRight: '0.5rem' }}>
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
                <div style={{ marginTop: '0.5rem', borderTop: '1px solid var(--glass-border)', paddingTop: '0.5rem' }}>
                    <h3 style={{ fontSize: '0.7rem', color: 'var(--accent-primary)', marginBottom: '0.3rem', letterSpacing: '1px' }}>SOURCE GENERADO</h3>
                    <div className="code-viewer" style={{ maxHeight: '200px' }}>
                        <pre><code>{distillResult}</code></pre>
                    </div>
                </div>
            ) : (
                <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                    <button 
                        className="btn-expand" 
                        style={{ width: '100%' }}
                        onClick={handleDistill}
                        disabled={isDistilling || selectedForDistill.length === 0}
                    >
                        {isDistilling ? 'CRISTALIZANDO...' : 'DISTILL SOURCE'}
                    </button>
                    <button 
                        className="btn-expand" 
                        style={{ width: '100%', background: 'rgba(128, 0, 255, 0.1)', borderColor: '#8000ff' }}
                        disabled={selectedForDistill.length === 0}
                        onClick={() => alert('Synthesis/Merge requires selecting a remote archive.')}
                    >
                        SYNARCHIC SYNTHESIS
                    </button>
                </div>
            )}
        </div>
    );
};

export default DistillPanel;
