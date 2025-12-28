import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Palette, Type, ChevronDown, Check } from 'lucide-react';
import './ThemeSelector.css';

const THEMES = {
  purple: {
    name: 'Violet',
    colors: {
      accent: '#818cf8',
      accentSecondary: '#a78bfa',
      accentTertiary: '#c084fc',
      rgb: '129, 140, 248',
    },
  },
  blue: {
    name: 'Bleu',
    colors: {
      accent: '#3b82f6',
      accentSecondary: '#60a5fa',
      accentTertiary: '#93c5fd',
      rgb: '59, 130, 246',
    },
  },
  green: {
    name: 'Vert',
    colors: {
      accent: '#10b981',
      accentSecondary: '#34d399',
      accentTertiary: '#6ee7b7',
      rgb: '16, 185, 129',
    },
  },
  pink: {
    name: 'Rose',
    colors: {
      accent: '#ec4899',
      accentSecondary: '#f472b6',
      accentTertiary: '#f9a8d4',
      rgb: '236, 72, 153',
    },
  },
  orange: {
    name: 'Orange',
    colors: {
      accent: '#f59e0b',
      accentSecondary: '#fbbf24',
      accentTertiary: '#fcd34d',
      rgb: '245, 158, 11',
    },
  },
  cyan: {
    name: 'Cyan',
    colors: {
      accent: '#06b6d4',
      accentSecondary: '#22d3ee',
      accentTertiary: '#67e8f9',
      rgb: '6, 182, 212',
    },
  },
};

const FONT_SIZES = {
  small: { name: 'Petit', value: '14px', multiplier: 0.875 },
  medium: { name: 'Moyen', value: '16px', multiplier: 1 },
  large: { name: 'Grand', value: '18px', multiplier: 1.125 },
  xlarge: { name: 'Très grand', value: '20px', multiplier: 1.25 },
};

const ThemeSelector = ({ currentTheme, currentFontSize, onThemeChange, onFontSizeChange }) => {
  const [isOpen, setIsOpen] = useState(false);
  const [activeTab, setActiveTab] = useState('color'); // 'color' or 'font'

  const applyTheme = (themeKey) => {
    const theme = THEMES[themeKey];
    if (!theme) return;

    const root = document.documentElement;
    root.style.setProperty('--accent-primary', theme.colors.accent);
    root.style.setProperty('--accent-secondary', theme.colors.accentSecondary);
    root.style.setProperty('--accent-tertiary', theme.colors.accentTertiary);
    root.style.setProperty('--accent-rgb', theme.colors.rgb);
    root.style.setProperty(
      '--accent-gradient',
      `linear-gradient(135deg, ${theme.colors.accent} 0%, ${theme.colors.accentSecondary} 50%, ${theme.colors.accentTertiary} 100%)`
    );
    root.style.setProperty(
      '--accent-gradient-subtle',
      `linear-gradient(135deg, ${theme.colors.accent}15 0%, ${theme.colors.accentSecondary}15 100%)`
    );
    root.style.setProperty('--accent-glow', `${theme.colors.accent}66`);

    onThemeChange(themeKey);
    localStorage.setItem('theme-color', themeKey);
  };

  const applyFontSize = (sizeKey) => {
    const size = FONT_SIZES[sizeKey];
    if (!size) return;

    document.documentElement.style.fontSize = size.value;
    onFontSizeChange(sizeKey);
    localStorage.setItem('font-size', sizeKey);
  };

  return (
    <div className="theme-selector-container">
      <button
        className="theme-selector-button"
        onClick={() => setIsOpen(!isOpen)}
        title="Personnaliser le thème"
      >
        <Palette size={20} />
      </button>

      <AnimatePresence>
        {isOpen && (
          <motion.div
            className="theme-selector-panel"
            initial={{ opacity: 0, y: -10, scale: 0.95 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -10, scale: 0.95 }}
            transition={{ duration: 0.2 }}
          >
            <div className="theme-selector-tabs">
              <button
                className={`theme-tab ${activeTab === 'color' ? 'active' : ''}`}
                onClick={() => setActiveTab('color')}
              >
                <Palette size={16} />
                Couleur
              </button>
              <button
                className={`theme-tab ${activeTab === 'font' ? 'active' : ''}`}
                onClick={() => setActiveTab('font')}
              >
                <Type size={16} />
                Police
              </button>
            </div>

            <div className="theme-selector-content">
              {activeTab === 'color' && (
                <div className="theme-colors-grid">
                  {Object.entries(THEMES).map(([key, theme]) => (
                    <button
                      key={key}
                      className={`theme-color-option ${currentTheme === key ? 'active' : ''}`}
                      onClick={() => applyTheme(key)}
                      title={theme.name}
                    >
                      <div
                        className="theme-color-preview"
                        style={{
                          background: `linear-gradient(135deg, ${theme.colors.accent} 0%, ${theme.colors.accentSecondary} 50%, ${theme.colors.accentTertiary} 100%)`,
                        }}
                      >
                        {currentTheme === key && (
                          <motion.div
                            initial={{ scale: 0 }}
                            animate={{ scale: 1 }}
                            className="theme-check"
                          >
                            <Check size={16} />
                          </motion.div>
                        )}
                      </div>
                      <span className="theme-color-name">{theme.name}</span>
                    </button>
                  ))}
                </div>
              )}

              {activeTab === 'font' && (
                <div className="theme-font-sizes">
                  {Object.entries(FONT_SIZES).map(([key, size]) => (
                    <button
                      key={key}
                      className={`theme-font-option ${currentFontSize === key ? 'active' : ''}`}
                      onClick={() => applyFontSize(key)}
                    >
                      <span className="font-size-label">{size.name}</span>
                      <span className="font-size-value" style={{ fontSize: size.value }}>
                        Aa
                      </span>
                      {currentFontSize === key && (
                        <motion.div
                          initial={{ scale: 0 }}
                          animate={{ scale: 1 }}
                          className="theme-check"
                        >
                          <Check size={16} />
                        </motion.div>
                      )}
                    </button>
                  ))}
                </div>
              )}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
};

export default ThemeSelector;

