import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { File, HardDrive, Clock, User, Video, Music } from 'lucide-react';
import './DownloadCardHover.css';

const DownloadCardHover = ({ download }) => {
  const [showHover, setShowHover] = useState(false);

  if (!download || download.status !== 'completed') {
    return null;
  }

  const formatFileSize = (bytes) => {
    if (!bytes) return 'N/A';
    const units = ['B', 'KB', 'MB', 'GB'];
    let size = bytes;
    let unitIndex = 0;
    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }
    return `${size.toFixed(1)} ${units[unitIndex]}`;
  };

  const formatDuration = (seconds) => {
    if (!seconds) return null;
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    }
    return `${minutes}m`;
  };

  return (
    <div
      className="download-card-hover-container"
      onMouseEnter={() => setShowHover(true)}
      onMouseLeave={() => setShowHover(false)}
    >
      <AnimatePresence>
        {showHover && (
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 10 }}
            className="download-card-hover"
          >
            {download.title && (
              <div className="hover-item">
                <File size={14} />
                <span className="hover-label">Titre:</span>
                <span className="hover-value">{download.title}</span>
              </div>
            )}
            {download.file_size && (
              <div className="hover-item">
                <HardDrive size={14} />
                <span className="hover-label">Taille:</span>
                <span className="hover-value">{formatFileSize(download.file_size)}</span>
              </div>
            )}
            {download.duration && (
              <div className="hover-item">
                <Clock size={14} />
                <span className="hover-label">Durée:</span>
                <span className="hover-value">{formatDuration(download.duration)}</span>
              </div>
            )}
            {download.author && (
              <div className="hover-item">
                <User size={14} />
                <span className="hover-label">Auteur:</span>
                <span className="hover-value">{download.author}</span>
              </div>
            )}
            {download.file_path && (
              <div className="hover-item hover-path">
                <File size={14} />
                <span className="hover-label">Chemin:</span>
                <span className="hover-value" title={download.file_path}>
                  {download.file_path.split('/').pop() || download.file_path}
                </span>
              </div>
            )}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
};

export default DownloadCardHover;

