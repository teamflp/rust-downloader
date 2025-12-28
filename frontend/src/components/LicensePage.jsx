import { useState, useEffect } from 'react';
import { ArrowLeft, FileText, Loader2, AlertCircle } from 'lucide-react';
import { motion } from 'framer-motion';
import './LicensePage.css';

const LicensePage = ({ onBack }) => {
  const [content, setContent] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    const fetchLicense = async () => {
      try {
        setLoading(true);
        setError(null);
        const response = await fetch('/api/license');
        
        if (!response.ok) {
          throw new Error(`Erreur ${response.status}: ${response.statusText}`);
        }

        const text = await response.text();
        setContent(text);
      } catch (err) {
        console.error('Error fetching license:', err);
        setError(err.message || 'Impossible de charger le contenu');
      } finally {
        setLoading(false);
      }
    };

    fetchLicense();
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
          <FileText size={24} />
          <h1>Licence MIT</h1>
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
          <pre className="license-content">{content}</pre>
        )}
      </div>
    </motion.div>
  );
};

export default LicensePage;

