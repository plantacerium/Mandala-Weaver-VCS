import React, { useState } from 'react';
import { useWorkspaceStore } from '../../lib/state/workspaceStore';
import { invoke } from '@tauri-apps/api/core';
import { isTauri } from '../../lib/tauri/isTauri';
import { distillSpatialQuick } from '../../lib/tauri/commands';
import type { DistilledArtifact } from '../../types/ontology';
import TemplateEditor from './TemplateEditor';
import '../../styles/panels/panels.css';

const DistillPanel: React.FC = () => {
    const { selectedForDistill, setSelectedForDistill, viewMode } = useWorkspaceStore();
    const [isDistilling, setIsDistilling] = useState(false);
    const [distillResult, setDistillResult] = useState<string | null>(null);

    // Active tabs: 'spatial' (Weaver Espacial SO2), 'lasso' (Lazo Manual), 'yaml' (Plantilla YAML)
    const [activeTab, setActiveTab] = useState<'spatial' | 'lasso' | 'yaml'>('spatial');

    // Spatial Weaver form state
    const [spatialAngleDeg, setSpatialAngleDeg] = useState<number>(45);
    const [spatialMaxRing, setSpatialMaxRing] = useState<number>(1);
    const [spatialSeedSymbol, setSpatialSeedSymbol] = useState<string>('main');
    const [spatialArtifact, setSpatialArtifact] = useState<DistilledArtifact | null>(null);
    const [spatialError, setSpatialError] = useState<string | null>(null);
    const [copied, setCopied] = useState<boolean>(false);

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
        setSpatialArtifact(null);
        setSpatialError(null);
    };

    const handleDistillManual = async () => {
        setIsDistilling(true);
        try {
            const monad_ids = selectedForDistill.map(m => m.id);
            if (isTauri()) {
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

    const handleDistillSpatial = async () => {
        setIsDistilling(true);
        setSpatialError(null);
        try {
            const seed = spatialSeedSymbol.trim() || (selectedForDistill[0] ? selectedForDistill[0].name : 'main');
            const artifact = await distillSpatialQuick(
                'spatial_distill',
                spatialAngleDeg,
                spatialMaxRing,
                [seed]
            );
            setSpatialArtifact(artifact);
        } catch (error) {
            console.error('Spatial distill failed', error);
            setSpatialError(String(error));
            setSpatialArtifact(null);
        } finally {
            setIsDistilling(false);
        }
    };

    const copyToClipboard = (text: string) => {
        navigator.clipboard.writeText(text);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };

    const handleExport = async () => {
        try {
            if (isTauri()) {
                await invoke('export_mandala_archive', { projectName: 'mandala-project', outputPath: '.' });
                alert('Mandala emanated to current directory (mandala.json)');
            }
        } catch (error) {
            console.error('Export failed', error);
        }
    };

    const handleImport = async () => {
        try {
            if (isTauri()) {
                await invoke('import_mandala_archive', { archivePath: './mandala.json' });
                window.location.reload();
            }
        } catch (error) {
            console.error('Import failed', error);
            alert('Import failed: ' + error);
        }
    };

    return (
        <div className="inspector" style={{ left: '2rem', right: 'auto', width: '460px', zIndex: 100, maxHeight: '88vh', overflowY: 'auto' }}>
            <header style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.5rem' }}>
                <h2>Weaver / Cristalización</h2>
                <button 
                    onClick={clearSelection} 
                    style={{ background: 'transparent', border: 'none', color: 'var(--text-dim)', cursor: 'pointer', fontFamily: 'monospace', fontSize: '1.25rem', padding: '0 0.5rem' }}
                >×</button>
            </header>
            
            <div style={{ display: 'flex', gap: '0.25rem', marginBottom: '0.75rem', background: 'rgba(0,0,0,0.3)', padding: '0.2rem', borderRadius: '4px' }}>
                <button 
                    onClick={() => setActiveTab('spatial')}
                    style={{
                        flex: 1,
                        padding: '0.35rem 0.2rem',
                        fontSize: '0.72rem',
                        background: activeTab === 'spatial' ? 'var(--accent-primary, #00e5ff)' : 'transparent',
                        color: activeTab === 'spatial' ? '#000' : 'var(--text-dim)',
                        border: 'none',
                        borderRadius: '3px',
                        cursor: 'pointer',
                        fontWeight: activeTab === 'spatial' ? 'bold' : 'normal',
                    }}
                >
                    🧭 Espacial SO(2)
                </button>
                <button 
                    onClick={() => setActiveTab('lasso')}
                    style={{
                        flex: 1,
                        padding: '0.35rem 0.2rem',
                        fontSize: '0.72rem',
                        background: activeTab === 'lasso' ? 'var(--accent-primary, #00e5ff)' : 'transparent',
                        color: activeTab === 'lasso' ? '#000' : 'var(--text-dim)',
                        border: 'none',
                        borderRadius: '3px',
                        cursor: 'pointer',
                        fontWeight: activeTab === 'lasso' ? 'bold' : 'normal',
                    }}
                >
                    Lazo ({selectedForDistill.length})
                </button>
                <button 
                    onClick={() => setActiveTab('yaml')}
                    style={{
                        flex: 1,
                        padding: '0.35rem 0.2rem',
                        fontSize: '0.72rem',
                        background: activeTab === 'yaml' ? 'var(--accent-primary, #00e5ff)' : 'transparent',
                        color: activeTab === 'yaml' ? '#000' : 'var(--text-dim)',
                        border: 'none',
                        borderRadius: '3px',
                        cursor: 'pointer',
                        fontWeight: activeTab === 'yaml' ? 'bold' : 'normal',
                    }}
                >
                    Plantilla YAML
                </button>
            </div>

            {activeTab === 'spatial' ? (
                <div>
                    <div style={{ marginBottom: '0.8rem', padding: '0.6rem', background: 'rgba(0, 229, 255, 0.04)', borderRadius: '6px', border: '1px solid rgba(0, 229, 255, 0.15)' }}>
                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.3rem' }}>
                            <span style={{ fontSize: '0.75rem', fontWeight: 'bold', color: 'var(--accent-primary)' }}>Rumbo Angular Target (θ)</span>
                            <span style={{ fontSize: '0.75rem', fontFamily: 'monospace', color: '#fff' }}>
                                {spatialAngleDeg}° ({(spatialAngleDeg * Math.PI / 180).toFixed(3)} rad)
                            </span>
                        </div>
                        <input 
                            type="range" 
                            min="0" 
                            max="360" 
                            step="1" 
                            value={spatialAngleDeg}
                            onChange={(e) => setSpatialAngleDeg(Number(e.target.value))}
                            style={{ width: '100%', cursor: 'pointer', accentColor: 'var(--accent-primary)' }}
                        />
                        <div style={{ display: 'flex', gap: '0.2rem', marginTop: '0.4rem', flexWrap: 'wrap' }}>
                            {[
                                { label: '0° Core', angle: 0 },
                                { label: '45° Math', angle: 45 },
                                { label: '90° UI', angle: 90 },
                                { label: '180° DB', angle: 180 },
                                { label: '270° Net', angle: 270 },
                            ].map(s => (
                                <button
                                    key={s.label}
                                    onClick={() => setSpatialAngleDeg(s.angle)}
                                    style={{
                                        fontSize: '0.65rem',
                                        padding: '0.2rem 0.4rem',
                                        borderRadius: '3px',
                                        background: spatialAngleDeg === s.angle ? 'rgba(0, 229, 255, 0.3)' : 'rgba(255,255,255,0.05)',
                                        border: '1px solid rgba(0, 229, 255, 0.2)',
                                        color: '#fff',
                                        cursor: 'pointer'
                                    }}
                                >
                                    {s.label}
                                </button>
                            ))}
                        </div>
                    </div>

                    <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '0.5rem', marginBottom: '0.8rem' }}>
                        <div>
                            <label style={{ fontSize: '0.7rem', color: 'var(--text-dim)', display: 'block', marginBottom: '0.2rem' }}>
                                Anillo Máximo ({spatialMaxRing})
                            </label>
                            <select 
                                value={spatialMaxRing} 
                                onChange={(e) => setSpatialMaxRing(Number(e.target.value))}
                                style={{ width: '100%', padding: '0.35rem', background: 'rgba(0,0,0,0.4)', border: '1px solid var(--border)', color: '#fff', borderRadius: '4px', fontSize: '0.75rem' }}
                            >
                                <option value={0}>Anillo 0 (Núcleo Inmutable)</option>
                                <option value={1}>Anillo 1 (Estable)</option>
                                <option value={2}>Anillo 2 (Experimental)</option>
                                <option value={5}>Anillo 5 (Todo el Mandala)</option>
                            </select>
                        </div>

                        <div>
                            <label style={{ fontSize: '0.7rem', color: 'var(--text-dim)', display: 'block', marginBottom: '0.2rem' }}>
                                Símbolo Semilla (Seed)
                            </label>
                            <input 
                                type="text" 
                                value={spatialSeedSymbol} 
                                onChange={(e) => setSpatialSeedSymbol(e.target.value)}
                                placeholder="main o función raíz"
                                style={{ width: '100%', padding: '0.35rem', background: 'rgba(0,0,0,0.4)', border: '1px solid var(--border)', color: '#fff', borderRadius: '4px', fontSize: '0.75rem', boxSizing: 'border-box' }}
                            />
                        </div>
                    </div>

                    {selectedForDistill.length > 0 && (
                        <div style={{ marginBottom: '0.6rem' }}>
                            <button 
                                onClick={() => setSpatialSeedSymbol(selectedForDistill[0].name)}
                                style={{ fontSize: '0.68rem', padding: '0.25rem 0.5rem', background: 'rgba(0, 255, 128, 0.1)', border: '1px solid rgba(0, 255, 128, 0.3)', color: '#00ff80', borderRadius: '3px', cursor: 'pointer' }}
                            >
                                🎯 Usar mónada seleccionada '{selectedForDistill[0].name}' como Semilla
                            </button>
                        </div>
                    )}

                    <button 
                        className="btn-expand" 
                        style={{ width: '100%', padding: '0.5rem', fontWeight: 'bold' }}
                        onClick={handleDistillSpatial}
                        disabled={isDistilling}
                    >
                        {isDistilling ? 'TEJIENDO EN SO(2)...' : 'TEJER ESPACIALMENTE (SO2)'}
                    </button>

                    {spatialError && (
                        <div style={{ marginTop: '0.75rem', padding: '0.5rem', background: 'rgba(255, 62, 131, 0.1)', border: '1px solid rgba(255, 62, 131, 0.4)', color: '#ff3e83', borderRadius: '4px', fontSize: '0.75rem' }}>
                            {spatialError}
                        </div>
                    )}

                    {spatialArtifact && (
                        <div style={{ marginTop: '0.8rem' }}>
                            {spatialArtifact.bridge_traces.length > 0 ? (
                                <div style={{ marginBottom: '0.6rem', padding: '0.5rem', background: 'rgba(255, 191, 0, 0.08)', border: '1px solid rgba(255, 191, 0, 0.4)', borderRadius: '4px' }}>
                                    <div style={{ display: 'flex', alignItems: 'center', gap: '0.4rem', color: '#ffbf00', fontSize: '0.75rem', fontWeight: 'bold' }}>
                                        <span>⚡ @mandala-bridge ({spatialArtifact.bridge_traces.length} adaptadores inyectados)</span>
                                    </div>
                                    <div style={{ marginTop: '0.4rem', display: 'flex', flexDirection: 'column', gap: '0.3rem' }}>
                                        {spatialArtifact.bridge_traces.map((bt, idx) => (
                                            <div key={idx} style={{ fontSize: '0.68rem', fontFamily: 'monospace', color: '#e0e0e0', padding: '0.3rem', background: 'rgba(0,0,0,0.3)', borderRadius: '3px' }}>
                                                <div><strong>{bt.caller_symbol}</strong> ➔ <span style={{ color: '#00e5ff' }}>{bt.bridge_alias}</span> ➔ <strong>{bt.target_symbol}</strong></div>
                                                <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: '2px', color: 'var(--text-dim)' }}>
                                                    <span>{bt.is_lossy ? '⚠️ Lossy adapter' : '🛡️ Lossless adapter'}</span>
                                                    <span>{bt.from_expected_contract.slice(0, 8)} ➔ {bt.to_actual_contract.slice(0, 8)}</span>
                                                </div>
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            ) : (
                                <div style={{ marginBottom: '0.6rem', padding: '0.4rem 0.6rem', background: 'rgba(0, 255, 128, 0.06)', border: '1px solid rgba(0, 255, 128, 0.25)', borderRadius: '4px', color: '#00ff80', fontSize: '0.72rem' }}>
                                    ✓ Coherencia directa total (sin colisión de contratos)
                                </div>
                            )}

                            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.3rem' }}>
                                <span style={{ fontSize: '0.7rem', color: 'var(--accent-primary)', letterSpacing: '1px' }}>
                                    ARTEFACTO ({spatialArtifact.traces.length} Mónadas)
                                </span>
                                <button
                                    onClick={() => copyToClipboard(spatialArtifact.assembled_source)}
                                    style={{ fontSize: '0.65rem', padding: '0.2rem 0.5rem', background: 'rgba(255,255,255,0.08)', border: '1px solid var(--border)', color: '#fff', borderRadius: '3px', cursor: 'pointer' }}
                                >
                                    {copied ? '✓ Copiado' : 'Copiar Código'}
                                </button>
                            </div>

                            <div className="code-viewer" style={{ maxHeight: '200px' }}>
                                <pre><code>{spatialArtifact.assembled_source}</code></pre>
                            </div>

                            {spatialArtifact.traces.length > 0 && (
                                <div style={{ marginTop: '0.5rem' }}>
                                    <span style={{ fontSize: '0.68rem', color: 'var(--text-dim)' }}>Traza angular de selección:</span>
                                    <div style={{ maxHeight: '100px', overflowY: 'auto', marginTop: '0.2rem', fontSize: '0.65rem', fontFamily: 'monospace' }}>
                                        {spatialArtifact.traces.map((t, idx) => (
                                            <div key={idx} style={{ display: 'flex', justifyContent: 'space-between', padding: '2px 0', borderBottom: '1px solid rgba(255,255,255,0.05)' }}>
                                                <span style={{ color: '#fff' }}>{t.symbol_name}</span>
                                                <span style={{ color: 'var(--text-dim)' }}>R{t.selected_ring} | Δθ: {t.angular_distance_rad.toFixed(3)} rad ({t.alternatives_considered} alt)</span>
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            )}
                        </div>
                    )}
                </div>
            ) : activeTab === 'yaml' ? (
                <TemplateEditor />
            ) : (
                <>
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
                            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.3rem' }}>
                                <h3 style={{ fontSize: '0.7rem', color: 'var(--accent-primary)', letterSpacing: '1px' }}>SOURCE GENERADO</h3>
                                <button
                                    onClick={() => copyToClipboard(distillResult)}
                                    style={{ fontSize: '0.65rem', padding: '0.2rem 0.5rem', background: 'rgba(255,255,255,0.08)', border: '1px solid var(--border)', color: '#fff', borderRadius: '3px', cursor: 'pointer' }}
                                >
                                    {copied ? '✓ Copiado' : 'Copiar Código'}
                                </button>
                            </div>
                            <div className="code-viewer" style={{ maxHeight: '200px' }}>
                                <pre><code>{distillResult}</code></pre>
                            </div>
                        </div>
                    ) : (
                        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                            <button 
                                className="btn-expand" 
                                style={{ width: '100%' }}
                                onClick={handleDistillManual}
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
                </>
            )}
        </div>
    );
};

export default DistillPanel;
