import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { isTauri } from '../../lib/tauri/isTauri';
import { distillSpatialRecipe } from '../../lib/tauri/commands';

interface TemplateInfo {
  name: string;
  filename: string;
  content: string;
}

interface ValidationResult {
  valid: boolean;
  name: string;
  version: string;
  vectors_count: number;
}

interface TemplateResult {
  source: string;
  monad_count: number;
  monad_names: string[];
  rings_included: string;
  output_file: string;
}

export const TemplateEditor: React.FC = () => {
  const [templates, setTemplates] = useState<TemplateInfo[]>([]);
  const [selectedTemplate, setSelectedTemplate] = useState<string>('spatial-simd');
  const [yamlContent, setYamlContent] = useState<string>('');
  const [validation, setValidation] = useState<ValidationResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [distillResult, setDistillResult] = useState<TemplateResult | null>(null);
  const [isProcessing, setIsProcessing] = useState(false);

  useEffect(() => {
    loadTemplates();
  }, []);

  const loadTemplates = async () => {
    const spatialPreset: TemplateInfo = {
      name: 'spatial-simd',
      filename: 'distill.yaml',
      content: `name: "motor_calculo_espacial"\ntarget_angle_rad: 0.785398 # 45 grados (Sector SIMD/Math)\nmax_ring: 1\nangular_policy: "GlobalTarget"\nring_penalty_rad: 0.05\nprelude: |\n  //! Artefacto destilado espacialmente por Mandala Weaver VCS\nseeds:\n  - symbol: "main"\nexternal_symbols:\n  - "println"`
    };

    try {
      if (isTauri()) {
        const list = await invoke<TemplateInfo[]>('list_templates');
        const merged = [spatialPreset, ...list.filter(t => t.name !== 'spatial-simd')];
        setTemplates(merged);
        setSelectedTemplate(merged[0].name);
        setYamlContent(merged[0].content);
      } else {
        // Fallback for browser mode
        const mock: TemplateInfo[] = [
          spatialPreset,
          {
            name: 'minimal',
            filename: 'minimal.yaml',
            content: `name: minimal\nversion: "0.1.0"\nrings:\n  type: Latest\nvectors:\n  - name: CORE\nexclude: []\nadapters: []\noutput:\n  filename: "minimal.rs"\n  structure: Flat\n  generate_modules: false`
          },
          {
            name: 'core-only',
            filename: 'core-only.yaml',
            content: `name: core-only\nversion: "0.1.0"\nrings:\n  type: Latest\nvectors:\n  - name: CORE\n    ring: 3\nexclude:\n  - pattern: "test_*"\nadapters: []\noutput:\n  filename: "lib/core.rs"\n  structure: Flat`
          }
        ];
        setTemplates(mock);
        setSelectedTemplate(mock[0].name);
        setYamlContent(mock[0].content);
      }
    } catch (err) {
      console.error('Failed to load templates:', err);
    }
  };

  const handleSelectTemplate = (name: string) => {
    setSelectedTemplate(name);
    const found = templates.find(t => t.name === name);
    if (found) {
      setYamlContent(found.content);
      setValidation(null);
      setError(null);
      setDistillResult(null);
    }
  };

  const handleValidate = async () => {
    setError(null);
    try {
      if (isTauri()) {
        const res = await invoke<ValidationResult>('validate_template_content', {
          templateContent: yamlContent,
        });
        setValidation(res);
      } else {
        setValidation({
          valid: true,
          name: selectedTemplate,
          version: '0.1.0',
          vectors_count: 1,
        });
      }
    } catch (err) {
      setError(String(err));
      setValidation(null);
    }
  };

  const handleDistill = async () => {
    setIsProcessing(true);
    setError(null);
    try {
      if (yamlContent.includes('target_angle_rad') || yamlContent.includes('seeds:')) {
        const artifact = await distillSpatialRecipe(yamlContent);
        setDistillResult({
          source: artifact.assembled_source,
          monad_count: artifact.traces.length,
          monad_names: artifact.traces.map(t => t.symbol_name),
          rings_included: String(Math.max(0, ...artifact.traces.map(t => t.selected_ring))),
          output_file: `${artifact.recipe_name || 'spatial'}.rs`,
        });
      } else if (isTauri()) {
        const res = await invoke<TemplateResult>('distill_from_template_content', {
          templateContent: yamlContent,
        });
        setDistillResult(res);
      } else {
        setDistillResult({
          source: `// Compilado desde plantilla YAML: ${selectedTemplate}\npub fn distilled_genesis() {\n    println!("Mandala Weaver Manifest Resolved");\n}`,
          monad_count: 8,
          monad_names: ['core_init', 'config_load', 'engine_start'],
          rings_included: '3',
          output_file: 'distilled_source.rs',
        });
      }
    } catch (err) {
      setError(String(err));
      setDistillResult(null);
    } finally {
      setIsProcessing(false);
    }
  };

  return (
    <div className="template-editor" style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
      <div style={{ display: 'flex', gap: '0.4rem', flexWrap: 'wrap' }}>
        {templates.map(t => (
          <button
            key={t.name}
            onClick={() => handleSelectTemplate(t.name)}
            style={{
              padding: '0.25rem 0.5rem',
              fontSize: '0.7rem',
              background: selectedTemplate === t.name ? 'var(--accent-primary, #00e5ff)' : 'rgba(255,255,255,0.05)',
              color: selectedTemplate === t.name ? '#000' : 'var(--text-main, #fff)',
              border: '1px solid rgba(255,255,255,0.1)',
              borderRadius: '4px',
              cursor: 'pointer',
              fontWeight: selectedTemplate === t.name ? 'bold' : 'normal',
            }}
          >
            {t.name}
          </button>
        ))}
      </div>

      <textarea
        value={yamlContent}
        onChange={(e) => setYamlContent(e.target.value)}
        spellCheck={false}
        style={{
          width: '100%',
          height: '140px',
          background: 'rgba(10, 10, 18, 0.8)',
          color: '#00ff88',
          fontFamily: 'monospace',
          fontSize: '0.75rem',
          border: '1px solid rgba(255, 255, 255, 0.1)',
          borderRadius: '4px',
          padding: '0.5rem',
          resize: 'vertical',
          boxSizing: 'border-box',
        }}
      />

      <div style={{ display: 'flex', gap: '0.5rem' }}>
        <button
          className="btn-expand"
          style={{ flex: 1, fontSize: '0.7rem', padding: '0.4rem' }}
          onClick={handleValidate}
        >
          VALIDAR YAML
        </button>
        <button
          className="btn-expand"
          style={{
            flex: 1,
            fontSize: '0.7rem',
            padding: '0.4rem',
            background: 'rgba(0, 229, 255, 0.15)',
            borderColor: 'var(--accent-primary)',
          }}
          onClick={handleDistill}
          disabled={isProcessing}
        >
          {isProcessing ? 'DESTILANDO...' : 'DESTILAR PLANTILLA'}
        </button>
      </div>

      {validation && (
        <div style={{ fontSize: '0.7rem', color: '#00ff88', padding: '0.3rem 0.5rem', background: 'rgba(0,255,136,0.1)', borderRadius: '4px' }}>
          ✓ Plantilla válida: <strong>{validation.name}</strong> (v{validation.version}) — {validation.vectors_count} vectores
        </div>
      )}

      {error && (
        <div style={{ fontSize: '0.7rem', color: '#ff3e83', padding: '0.3rem 0.5rem', background: 'rgba(255,62,131,0.1)', borderRadius: '4px', whiteSpace: 'pre-wrap' }}>
          ⚠ {error}
        </div>
      )}

      {distillResult && (
        <div style={{ borderTop: '1px solid var(--glass-border)', paddingTop: '0.5rem' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.7rem', color: 'var(--accent-primary)', marginBottom: '0.3rem' }}>
            <span>SALIDA: {distillResult.output_file}</span>
            <span>{distillResult.monad_count} Mónadas</span>
          </div>
          <div className="code-viewer" style={{ maxHeight: '180px', overflowY: 'auto' }}>
            <pre style={{ margin: 0 }}><code>{distillResult.source}</code></pre>
          </div>
        </div>
      )}
    </div>
  );
};

export default TemplateEditor;
