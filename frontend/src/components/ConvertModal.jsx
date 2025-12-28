import { useState, useEffect } from 'react';
import { X, RefreshCw, FileVideo, Music } from 'lucide-react';
import { downloadAPI } from '../api/client';
import './ConvertModal.css';

const ConvertModal = ({ download, isOpen, onClose, onUpdate }) => {
  const [selectedFormat, setSelectedFormat] = useState('');
  const [keepOriginal, setKeepOriginal] = useState(true);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  useEffect(() => {
    if (isOpen && download?.file_path) {
      // Detect file type from extension
      const extension = download.file_path.split('.').pop()?.toLowerCase();
      const isVideo = ['mp4', 'webm', 'mkv', 'avi', 'mov', 'flv', 'm4v'].includes(extension);
      
      // Set default format based on file type
      if (isVideo) {
        setSelectedFormat('mp3'); // Default to audio extraction for video
      } else {
        setSelectedFormat('mp3'); // Default for audio files
      }
      setKeepOriginal(true);
      setError(null);
    }
  }, [isOpen, download]);

  const getAvailableFormats = () => {
    if (!download?.file_path) return [];
    
    const extension = download.file_path.split('.').pop()?.toLowerCase();
    const isVideo = ['mp4', 'webm', 'mkv', 'avi', 'mov', 'flv', 'm4v'].includes(extension);
    
    if (isVideo) {
      return [
        { value: 'mp4', label: 'MP4 (Vidéo)', icon: <FileVideo size={16} />, description: 'Format vidéo universel' },
        { value: 'webm', label: 'WebM (Vidéo)', icon: <FileVideo size={16} />, description: 'Format web optimisé' },
        { value: 'mkv', label: 'MKV (Vidéo)', icon: <FileVideo size={16} />, description: 'Conteneur multimédia' },
        { value: 'mp3', label: 'MP3 (Audio)', icon: <Music size={16} />, description: 'Extraire l\'audio' },
      ];
    } else {
      return [
        { value: 'mp3', label: 'MP3', icon: <Music size={16} />, description: 'Format audio compressé' },
        { value: 'wav', label: 'WAV', icon: <Music size={16} />, description: 'Format audio non compressé' },
        { value: 'flac', label: 'FLAC', icon: <Music size={16} />, description: 'Format audio sans perte' },
        { value: 'm4a', label: 'M4A', icon: <Music size={16} />, description: 'Format Apple' },
        { value: 'aac', label: 'AAC', icon: <Music size={16} />, description: 'Format audio avancé' },
      ];
    }
  };

  const handleConvert = async () => {
    if (!selectedFormat) {
      setError('Veuillez sélectionner un format de conversion');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const updatedDownload = await downloadAPI.convertDownload(
        download.id,
        selectedFormat,
        keepOriginal
      );
      
      if (onUpdate) {
        onUpdate(updatedDownload);
      }
      
      onClose();
    } catch (err) {
      console.error('Conversion failed:', err);
      let errorMessage = 'Erreur lors de la conversion';
      
      if (err.response?.data?.message) {
        errorMessage = err.response.data.message;
      } else if (err.response?.data?.error) {
        errorMessage = err.response.data.error;
      } else if (err.message) {
        errorMessage = err.message;
      }
      
      // Vérifier si ffmpeg n'est pas installé
      if (errorMessage.includes('ffmpeg') || errorMessage.includes('Failed to start') || errorMessage.includes('spawn')) {
        errorMessage = 'ffmpeg n\'est pas installé ou n\'est pas dans le PATH. Veuillez installer ffmpeg pour utiliser la conversion.';
      }
      
      // Vérifier si le fichier n'existe pas
      if (errorMessage.includes('does not exist') || errorMessage.includes('n\'existe pas')) {
        errorMessage = 'Le fichier source n\'existe plus. Il a peut-être été déplacé ou supprimé.';
      }
      
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  if (!isOpen) return null;

  const availableFormats = getAvailableFormats();

  return (
    <div className="convert-modal-overlay" onClick={onClose}>
      <div className="convert-modal" onClick={(e) => e.stopPropagation()}>
        <div className="convert-modal-header">
          <h3>Convertir le fichier</h3>
          <button className="btn-close" onClick={onClose}>
            <X size={20} />
          </button>
        </div>

        <div className="convert-modal-content">
          {download?.title && (
            <div className="convert-file-info">
              <h4>{download.title}</h4>
              <p className="file-path">{download.file_path?.split('/').pop()}</p>
            </div>
          )}

          <div className="convert-formats">
            <label>Format de destination</label>
            <div className="formats-grid">
              {availableFormats.map(format => (
                <button
                  key={format.value}
                  className={`format-option ${selectedFormat === format.value ? 'selected' : ''}`}
                  onClick={() => setSelectedFormat(format.value)}
                  disabled={loading}
                >
                  <div className="format-icon">{format.icon}</div>
                  <div className="format-info">
                    <div className="format-name">{format.label}</div>
                    <div className="format-description">{format.description}</div>
                  </div>
                </button>
              ))}
            </div>
          </div>

          <div className="convert-options">
            <label className="checkbox-label">
              <input
                type="checkbox"
                checked={keepOriginal}
                onChange={(e) => setKeepOriginal(e.target.checked)}
                disabled={loading}
              />
              <span>Conserver le fichier original</span>
            </label>
          </div>

          {error && (
            <div className="convert-error">
              {error}
            </div>
          )}
        </div>

        <div className="convert-modal-actions">
          <button
            className="btn-secondary"
            onClick={onClose}
            disabled={loading}
          >
            Annuler
          </button>
          <button
            className="btn-primary"
            onClick={handleConvert}
            disabled={loading || !selectedFormat}
          >
            {loading ? (
              <>
                <RefreshCw size={16} className="animate-spin" />
                Conversion...
              </>
            ) : (
              'Convertir'
            )}
          </button>
        </div>
      </div>
    </div>
  );
};

export default ConvertModal;

