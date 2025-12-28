import { useState, useEffect } from 'react';
import { ArrowLeft, Shield, Loader2, AlertCircle } from 'lucide-react';
import { motion } from 'framer-motion';
import './DisclaimerPage.css';

const DisclaimerPage = ({ onBack }) => {
  const [content, setContent] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    const fetchDisclaimer = async () => {
      try {
        setLoading(true);
        setError(null);
        const response = await fetch('/api/disclaimer');
        
        if (!response.ok) {
          throw new Error(`Erreur ${response.status}: ${response.statusText}`);
        }

        const text = await response.text();
        setContent(text);
      } catch (err) {
        console.error('Error fetching disclaimer:', err);
        setError(err.message || 'Impossible de charger le contenu');
      } finally {
        setLoading(false);
      }
    };

    fetchDisclaimer();
  }, []);

  return (
    <motion.div
      className="legal-page"
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      transition={{ duration: 0.3 }}
    >
      <div className="legal-page-header">
        <button className="back-button" onClick={onBack} title="Retour">
          <ArrowLeft size={20} />
          Retour
        </button>
        <div className="legal-page-title">
          <Shield size={24} />
          <h1>Conditions d'utilisation et Avertissement</h1>
        </div>
      </div>

      <div className="legal-page-content">
        {loading ? (
          <div className="legal-page-loading">
            <Loader2 size={32} className="animate-spin" />
            <p>Chargement...</p>
          </div>
        ) : error ? (
          <div className="legal-page-error">
            <AlertCircle size={32} />
            <h2>Erreur de chargement</h2>
            <p>{error}</p>
            <p className="error-hint">
              Vérifiez que le backend est démarré sur le port 9000.
            </p>
          </div>
        ) : (
          <div
            className="markdown-content"
            dangerouslySetInnerHTML={{ __html: formatMarkdown(content) }}
          />
        )}
      </div>
    </motion.div>
  );
};

// Simple markdown to HTML converter
function formatMarkdown(text) {
  if (!text) return '';

  return text
    // Headers
    .replace(/^### (.*$)/gim, '<h3>$1</h3>')
    .replace(/^## (.*$)/gim, '<h2>$1</h2>')
    .replace(/^# (.*$)/gim, '<h1>$1</h1>')
    // Bold
    .replace(/\*\*(.+?)\*\*/gim, '<strong>$1</strong>')
    // Lists
    .replace(/^\* (.+)$/gim, '<li>$1</li>')
    .replace(/^(\d+)\. (.+)$/gim, '<li>$2</li>')
    // Line breaks
    .replace(/\n\n/gim, '</p><p>')
    .replace(/\n/gim, '<br>')
    // Wrap in paragraphs
    .split(/(<h[1-6]>.*?<\/h[1-6]>|<li>.*?<\/li>)/)
    .map((chunk) => {
      if (chunk.match(/^<(h[1-6]|li)/)) return chunk;
      if (chunk.trim()) return `<p>${chunk.trim()}</p>`;
      return '';
    })
    .join('')
    // Wrap consecutive <li> in <ul>
    .replace(/(<li>.*?<\/li>(\s*<li>.*?<\/li>)*)/gim, '<ul>$1</ul>')
    // Clean up empty paragraphs
    .replace(/<p>\s*<\/p>/gim, '')
    // Blockquotes
    .replace(/^> (.*$)/gim, '<blockquote>$1</blockquote>');
}

export default DisclaimerPage;

