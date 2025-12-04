import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import DownloadCard from './DownloadCard';
import './DownloadList.css';

const DownloadList = ({ downloads, onDelete }) => {
  const [filter, setFilter] = useState('all');

  const filteredDownloads = downloads.filter(download => {
    if (filter === 'all') return true;
    if (filter === 'active') return download.status === 'downloading' || download.status === 'processing' || download.status === 'pending';
    if (filter === 'completed') return download.status === 'completed';
    if (filter === 'failed') return download.status === 'failed';
    return true;
  });

  const getFilterCount = (filterType) => {
    if (filterType === 'all') return downloads.length;
    if (filterType === 'active') return downloads.filter(d => d.status === 'downloading' || d.status === 'processing' || d.status === 'pending').length;
    if (filterType === 'completed') return downloads.filter(d => d.status === 'completed').length;
    if (filterType === 'failed') return downloads.filter(d => d.status === 'failed').length;
    return 0;
  };

  return (
    <div className="download-list-container">
      <div className="list-header">
        <h3>Téléchargements</h3>
        <div className="filter-tabs">
          <button
            className={`filter-tab ${filter === 'all' ? 'active' : ''}`}
            onClick={() => setFilter('all')}
          >
            Tous
            <span className="filter-count">{getFilterCount('all')}</span>
          </button>
          <button
            className={`filter-tab ${filter === 'active' ? 'active' : ''}`}
            onClick={() => setFilter('active')}
          >
            En cours
            <span className="filter-count">{getFilterCount('active')}</span>
          </button>
          <button
            className={`filter-tab ${filter === 'completed' ? 'active' : ''}`}
            onClick={() => setFilter('completed')}
          >
            Terminés
            <span className="filter-count">{getFilterCount('completed')}</span>
          </button>
          <button
            className={`filter-tab ${filter === 'failed' ? 'active' : ''}`}
            onClick={() => setFilter('failed')}
          >
            Échoués
            <span className="filter-count">{getFilterCount('failed')}</span>
          </button>
        </div>
      </div>

      <div className="downloads-grid">
        <AnimatePresence mode="popLayout">
          {filteredDownloads.length === 0 ? (
            <motion.div
              className="empty-state"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
            >
              <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3V10" />
              </svg>
              <h4>Aucun téléchargement</h4>
              <p className="text-muted">
                {filter === 'all' 
                  ? 'Commencez par ajouter un nouveau téléchargement ci-dessus'
                  : `Aucun téléchargement ${filter === 'active' ? 'en cours' : filter === 'completed' ? 'terminé' : 'échoué'}`
                }
              </p>
            </motion.div>
          ) : (
            filteredDownloads.map(download => (
              <DownloadCard
                key={download.id}
                download={download}
                onDelete={onDelete}
              />
            ))
          )}
        </AnimatePresence>
      </div>
    </div>
  );
};

export default DownloadList;
