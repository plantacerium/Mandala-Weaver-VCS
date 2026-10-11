import React, { useState } from 'react';
import type { ScanMode } from '../../types/synarchy';

interface AddProjectDialogProps {
  onAdd: (path: string, name: string, scanMode: ScanMode) => Promise<void> | void;
  onClose: () => void;
}

export const AddProjectDialog: React.FC<AddProjectDialogProps> = ({
  onAdd,
  onClose,
}) => {
  const [path, setPath] = useState('');
  const [name, setName] = useState('');
  const [scanMode, setScanMode] = useState<ScanMode>('mandala');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!path.trim()) return;

    setIsSubmitting(true);
    setErrorMessage(null);
    try {
      await onAdd(path.trim(), name.trim() || path.trim().split(/[/\\]/).pop() || 'Unnamed', scanMode);
    } catch (err: any) {
      setErrorMessage(err?.message || String(err));
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog" onClick={(e) => e.stopPropagation()} style={{ maxWidth: '480px' }}>
        <h2>Añadir Proyecto a Synarchy</h2>

        {errorMessage && (
          <div style={{
            padding: '10px 14px',
            marginBottom: '16px',
            borderRadius: '8px',
            background: 'rgba(255, 62, 131, 0.15)',
            border: '1px solid #ff3e83',
            color: '#ff85aa',
            fontSize: '0.8rem',
            lineHeight: '1.4'
          }}>
            <div>{errorMessage}</div>
            {errorMessage.includes('Git') && (
              <div style={{ marginTop: '8px' }}>
                <button
                  type="button"
                  onClick={() => {
                    setScanMode('mandala');
                    setErrorMessage(null);
                  }}
                  style={{
                    background: 'rgba(0, 229, 255, 0.2)',
                    border: '1px solid #00e5ff',
                    color: '#00e5ff',
                    borderRadius: '6px',
                    padding: '5px 10px',
                    fontSize: '0.75rem',
                    cursor: 'pointer',
                    fontWeight: 500
                  }}
                >
                  Cambiar a Mandala Scan y escanear
                </button>
              </div>
            )}
          </div>
        )}

        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label>Ruta del Proyecto (Path)</label>
            <input
              type="text"
              value={path}
              onChange={(e) => {
                setPath(e.target.value);
                setErrorMessage(null);
              }}
              placeholder="Ej: I:\GITHUB\Mi-Proyecto o /home/user/project"
              autoFocus
              required
            />
          </div>

          <div className="form-group">
            <label>Nombre a Mostrar (opcional)</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Auto-detectado del directorio"
            />
          </div>

          <div className="form-group">
            <label>Modo de Escaneo</label>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '10px', marginTop: '6px' }}>
              <div
                onClick={() => {
                  setScanMode('mandala');
                  setErrorMessage(null);
                }}
                style={{
                  padding: '12px',
                  borderRadius: '10px',
                  border: scanMode === 'mandala' ? '1.5px solid var(--accent-primary, #00e5ff)' : '1px solid rgba(255,255,255,0.1)',
                  background: scanMode === 'mandala' ? 'rgba(0, 229, 255, 0.12)' : 'rgba(255,255,255,0.03)',
                  boxShadow: scanMode === 'mandala' ? '0 0 12px rgba(0, 229, 255, 0.2)' : 'none',
                  cursor: 'pointer',
                  display: 'flex',
                  flexDirection: 'column',
                  gap: '6px',
                  transition: 'all 0.2s ease'
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontWeight: 600, fontSize: '0.85rem', color: scanMode === 'mandala' ? '#00e5ff' : '#fff' }}>
                  <span>🌀</span>
                  <span>Mandala Scan</span>
                </div>
                <div style={{ fontSize: '0.7rem', color: 'rgba(255,255,255,0.65)', lineHeight: '1.3' }}>
                  Escaneo nativo de archivos y AST. Ideal para cualquier carpeta o proyecto <strong>sin Git</strong>.
                </div>
              </div>

              <div
                onClick={() => {
                  setScanMode('git');
                  setErrorMessage(null);
                }}
                style={{
                  padding: '12px',
                  borderRadius: '10px',
                  border: scanMode === 'git' ? '1.5px solid var(--accent-primary, #00e5ff)' : '1px solid rgba(255,255,255,0.1)',
                  background: scanMode === 'git' ? 'rgba(0, 229, 255, 0.12)' : 'rgba(255,255,255,0.03)',
                  boxShadow: scanMode === 'git' ? '0 0 12px rgba(0, 229, 255, 0.2)' : 'none',
                  cursor: 'pointer',
                  display: 'flex',
                  flexDirection: 'column',
                  gap: '6px',
                  transition: 'all 0.2s ease'
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontWeight: 600, fontSize: '0.85rem', color: scanMode === 'git' ? '#00e5ff' : '#fff' }}>
                  <span>🐙</span>
                  <span>Git Scan</span>
                </div>
                <div style={{ fontSize: '0.7rem', color: 'rgba(255,255,255,0.65)', lineHeight: '1.3' }}>
                  Repositorio Git. Lee ramas, historial y commits estructurados. Requiere carpeta <code>.git</code>.
                </div>
              </div>
            </div>
          </div>

          <div className="dialog-actions">
            <button type="button" onClick={onClose} className="btn-cancel" disabled={isSubmitting}>
              Cancelar
            </button>
            <button type="submit" className="btn-primary" disabled={!path.trim() || isSubmitting}>
              {isSubmitting ? 'Escaneando...' : 'Añadir y Escanear'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

export default AddProjectDialog;