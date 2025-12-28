import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, Save, FileText, User, StickyNote } from 'lucide-react';
import { downloadAPI } from '../api/client';
import './EditMetadataModal.css';

const EditMetadataModal = ({ download, isOpen, onClose, onSave }) => {
  const [title, setTitle] = useState('');
  const [author, setAuthor] = useState('');
  const [notes, setNotes] = useState('');
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState(null);

  useEffect(() => {
    if (download && isOpen) {
      setTitle(download.title || '');
      setAuthor(download.author || '');
      setNotes(download.notes || '');
      setError(null);
    }
  }, [download, isOpen]);

  const handleSave = async () => {
    if (!download) return;

    setIsSaving(true);
    setError(null);

    try {
      const updated = await downloadAPI.updateDownloadMetadata(download.id, {
        title: title.trim() || null,
        author: author.trim() || null,
        notes: notes.trim() || null,
      });

      if (onSave) {
        onSave(updated);
      }
      onClose();
    } catch (err) {
      setError(err.message || 'Erreur lors de la sauvegarde des métadonnées');
      console.error('Error updating metadata:', err);
    } finally {
      setIsSaving(false);
    }
  };

  const handleCancel = () => {
    setTitle(download?.title || '');
    setAuthor(download?.author || '');
    setNotes(download?.notes || '');
    setError(null);
    onClose();
  };

  if (!isOpen || !download) return null;

  return (
    <AnimatePresence>
      <div className="modal-overlay" onClick={handleCancel}>
        <motion.div
          className="edit-metadata-modal"
          initial={{ opacity: 0, scale: 0.9, y: 20 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.9, y: 20 }}
          transition={{ duration: 0.2 }}
          onClick={(e) => e.stopPropagation()}
        >
          <div className="modal-header">
            <h2>Éditer les métadonnées</h2>
            <button
              className="btn-close-modal"
              onClick={handleCancel}
              title="Fermer"
            >
              <X size={20} />
            </button>
          </div>

          <div className="modal-body">
            {error && (
              <div className="error-message">
                {error}
              </div>
            )}

            <div className="form-group">
              <label htmlFor="edit-title">
                <FileText size={16} />
                Titre
              </label>
              <input
                id="edit-title"
                type="text"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                placeholder="Titre du média"
                maxLength={200}
              />
            </div>

            <div className="form-group">
              <label htmlFor="edit-author">
                <User size={16} />
                Auteur
              </label>
              <input
                id="edit-author"
                type="text"
                value={author}
                onChange={(e) => setAuthor(e.target.value)}
                placeholder="Nom de l'auteur ou de la chaîne"
                maxLength={100}
              />
            </div>

            <div className="form-group">
              <label htmlFor="edit-notes">
                <StickyNote size={16} />
                Notes personnelles
              </label>
              <textarea
                id="edit-notes"
                value={notes}
                onChange={(e) => setNotes(e.target.value)}
                placeholder="Ajoutez vos notes personnelles ici..."
                rows={4}
                maxLength={1000}
              />
              <div className="char-count">
                {notes.length}/1000 caractères
              </div>
            </div>
          </div>

          <div className="modal-footer">
            <button
              className="btn-secondary"
              onClick={handleCancel}
              disabled={isSaving}
            >
              Annuler
            </button>
            <button
              className="btn-primary"
              onClick={handleSave}
              disabled={isSaving}
            >
              {isSaving ? (
                <>
                  <span className="spinner"></span>
                  Enregistrement...
                </>
              ) : (
                <>
                  <Save size={16} />
                  Enregistrer
                </>
              )}
            </button>
          </div>
        </motion.div>
      </div>
    </AnimatePresence>
  );
};

export default EditMetadataModal;

