import { useState } from 'react';
import { motion } from 'framer-motion';
import { Video, Music, Music2, Link2, Download, Loader2 } from 'lucide-react';
import './DownloadForm.css';

const DownloadForm = ({ onSubmit }) => {
  const [url, setUrl] = useState('');
  const [downloadType, setDownloadType] = useState('video');
  const [format, setFormat] = useState('');
  const [downloadPlaylist, setDownloadPlaylist] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState('');

  const handleSubmit = async (e) => {
    e.preventDefault();
    
    if (!url.trim()) {
      setError('Please enter a valid URL');
      return;
    }

    setIsSubmitting(true);
    setError('');

    try {
      await onSubmit({
        url: url.trim(),
        type: downloadType,
        format: format || undefined,
        download_playlist: downloadPlaylist,
      });
      
      // Reset form on success
      setUrl('');
      setFormat('');
    } catch (err) {
      setError(err.message || 'Failed to start download');
    } finally {
      setIsSubmitting(false);
    }
  };

  const getFormatOptions = () => {
    switch (downloadType) {
      case 'video':
        return ['mp4', 'webm', 'mkv'];
      case 'audio':
      case 'instrumental':
        return ['mp3', 'wav', 'm4a', 'flac'];
      default:
        return [];
    }
  };

  const downloadTypes = [
    { value: 'video', label: 'Vidéo', icon: Video },
    { value: 'audio', label: 'Audio', icon: Music },
    { value: 'instrumental', label: 'Instrumental', icon: Music2 },
  ];

  return (
    <motion.div
      className="download-form-container"
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
    >
      <div className="form-header">
        <h2>Télécharger un Média</h2>
        <p className="text-muted">Collez l'URL de votre vidéo ou audio préféré</p>
      </div>

      <form onSubmit={handleSubmit} className="download-form">
        <div className="form-group">
          <label htmlFor="url" className="form-label">
            URL du Média
          </label>
          <div className="input-wrapper">
            
            <input
              id="url"
              type="text"
              className="input input-url"
              placeholder="https://youtube.com/watch?v=..."
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              disabled={isSubmitting}
            />
          </div>
        </div>

        <div className="form-group">
          <label className="form-label">
            Type de Téléchargement
          </label>
          <div className="toggle-group">
            {downloadTypes.map((type) => {
              const IconComponent = type.icon;
              return (
                <button
                  key={type.value}
                  type="button"
                  className={`toggle-button ${downloadType === type.value ? 'active' : ''}`}
                  onClick={() => {
                    setDownloadType(type.value);
                    setFormat('');
                  }}
                  disabled={isSubmitting}
                >
                  <IconComponent className="toggle-icon" size={20} />
                  <span className="toggle-label">{type.label}</span>
                </button>
              );
            })}
          </div>
        </div>

        <div className="form-group">
          <label className="form-label">
            Format
          </label>
          <div className="toggle-group format-group">
            <button
              type="button"
              className={`toggle-button ${format === '' ? 'active' : ''}`}
              onClick={() => setFormat('')}
              disabled={isSubmitting}
            >
              Par défaut
            </button>
            {getFormatOptions().map((fmt) => (
              <button
                key={fmt}
                type="button"
                className={`toggle-button ${format === fmt ? 'active' : ''}`}
                onClick={() => setFormat(fmt)}
                disabled={isSubmitting}
              >
                {fmt.toUpperCase()}
              </button>
            ))}
          </div>
        </div>

        <div className="form-group">
          <label className="checkbox-container">
            <input
              type="checkbox"
              checked={downloadPlaylist}
              onChange={(e) => setDownloadPlaylist(e.target.checked)}
              disabled={isSubmitting}
              className="checkbox-input"
            />
            <span className="checkbox-label">
              Télécharger toute la playlist (si l'URL en contient une)
            </span>
          </label>
        </div>

        {error && (
          <motion.div
            className="error-message"
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor">
              <circle cx="12" cy="12" r="10" strokeWidth={2} />
              <line x1="12" y1="8" x2="12" y2="12" strokeWidth={2} strokeLinecap="round" />
              <line x1="12" y1="16" x2="12.01" y2="16" strokeWidth={2} strokeLinecap="round" />
            </svg>
            {error}
          </motion.div>
        )}

        <motion.button
          type="submit"
          className="btn btn-primary btn-submit"
          disabled={isSubmitting}
          whileHover={{ scale: 1.02 }}
          whileTap={{ scale: 0.98 }}
        >
          {isSubmitting ? (
            <>
              <Loader2 className="animate-spin" size={20} />
              Démarrage...
            </>
          ) : (
            <>
              <Download size={20} />
              Télécharger
            </>
          )}
        </motion.button>
      </form>
    </motion.div>
  );
};

export default DownloadForm;
