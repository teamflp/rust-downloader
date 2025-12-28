import { motion } from 'framer-motion';
import { Trash2, Tag, Star, X, Download } from 'lucide-react';
import './BulkActionBar.css';

const BulkActionBar = ({ selectedCount, onDelete, onTag, onFavorite, onClearSelection }) => {
  if (selectedCount === 0) return null;

  return (
    <motion.div
      initial={{ y: 100, opacity: 0 }}
      animate={{ y: 0, opacity: 1 }}
      exit={{ y: 100, opacity: 0 }}
      className="bulk-action-bar"
    >
      <div className="bulk-action-bar-content">
        <div className="bulk-action-info">
          <span className="bulk-action-count">{selectedCount}</span>
          <span className="bulk-action-text">
            {selectedCount === 1 ? 'élément sélectionné' : 'éléments sélectionnés'}
          </span>
        </div>
        
        <div className="bulk-action-buttons">
          {onFavorite && (
            <button
              className="bulk-action-btn"
              onClick={onFavorite}
              title="Ajouter aux favoris"
              aria-label="Ajouter aux favoris"
            >
              <Star size={18} />
              <span>Favoris</span>
            </button>
          )}
          
          {onTag && (
            <button
              className="bulk-action-btn"
              onClick={onTag}
              title="Ajouter des tags"
              aria-label="Ajouter des tags"
            >
              <Tag size={18} />
              <span>Tags</span>
            </button>
          )}
          
          {onDelete && (
            <button
              className="bulk-action-btn bulk-action-btn-danger"
              onClick={onDelete}
              title="Supprimer les éléments sélectionnés"
              aria-label="Supprimer"
            >
              <Trash2 size={18} />
              <span>Supprimer</span>
            </button>
          )}
          
          <button
            className="bulk-action-btn bulk-action-btn-close"
            onClick={onClearSelection}
            title="Annuler la sélection"
            aria-label="Annuler la sélection"
          >
            <X size={18} />
          </button>
        </div>
      </div>
    </motion.div>
  );
};

export default BulkActionBar;

