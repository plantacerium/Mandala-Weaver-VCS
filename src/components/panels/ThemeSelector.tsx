import { useState, useEffect } from 'react';

type Theme = 'dark' | 'light' | 'love' | 'golden' | 'ancient' | 'futuristic';

const themes: { id: Theme; label: string; color: string }[] = [
  { id: 'love', label: 'Love', color: '#ff2d55' },
  { id: 'light', label: 'Light', color: '#0284c7' },
  { id: 'dark', label: 'Dark', color: '#00e5ff' },
  { id: 'ancient', label: 'Ancient', color: '#c9a227' },
  { id: 'futuristic', label: 'Future', color: '#00ffcc' },
  { id: 'golden', label: 'Golden', color: '#ffd700' },
];

export default function ThemeSelector() {
  const [isOpen, setIsOpen] = useState(false);
  const [currentTheme, setCurrentTheme] = useState<Theme>('dark');

  useEffect(() => {
    const saved = localStorage.getItem('mandala-theme') as Theme;
    if (saved && themes.some(t => t.id === saved)) {
      setCurrentTheme(saved);
      document.documentElement.setAttribute('data-theme', saved);
    }
  }, []);

  const selectTheme = (theme: Theme) => {
    setCurrentTheme(theme);
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem('mandala-theme', theme);
    setIsOpen(false);
  };

  const current = themes.find(t => t.id === currentTheme);

  return (
    <div className="theme-selector">
      <button
        className="theme-btn"
        onClick={() => setIsOpen(!isOpen)}
        style={{
          '--theme-color': current?.color,
          borderColor: `rgba(${hexToRgb(current?.color || '#00e5ff')}, 0.3)`
        } as React.CSSProperties}
      >
        <span className="theme-indicator" style={{ background: current?.color }} />
        <span className="theme-label">{current?.label}</span>
        <svg className={`chevron ${isOpen ? 'open' : ''}`} viewBox="0 0 24 24" width="16" height="16">
          <path d="M7 10l5 5 5-5" stroke="currentColor" strokeWidth="2" fill="none" />
        </svg>
      </button>

      {isOpen && (
        <div className="theme-dropdown">
          {themes.map((theme) => (
            <button
              key={theme.id}
              className={`theme-option ${currentTheme === theme.id ? 'active' : ''}`}
              onClick={() => selectTheme(theme.id as Theme)}
              style={{ '--option-color': theme.color } as React.CSSProperties}
            >
              <span className="theme-indicator" style={{ background: theme.color }} />
              <span>{theme.label}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

function hexToRgb(hex: string): string {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  return result
    ? `${parseInt(result[1], 16)}, ${parseInt(result[2], 16)}, ${parseInt(result[3], 16)}`
    : '0, 229, 255';
}