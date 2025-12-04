import { motion } from 'framer-motion';
import './DownloadCard.css';

const DownloadCard = ({ download, onDelete }) => {
  const getStatusIcon = (status) => {
    switch (status) {
      case 'pending':
        return (
          <svg className="status-icon pending" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <circle cx="12" cy="12" r="10" strokeWidth={2} />
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6l4 2" />
          </svg>
        );
      case 'downloading':
      case 'processing':
        return (
          <svg className="status-icon downloading animate-spin" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <circle cx="12" cy="12" r="10" strokeWidth={3} strokeDasharray="60" strokeDashoffset="30" />
          </svg>
        );
      case 'completed':
        return (
          <svg className="status-icon completed" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <circle cx="12" cy="12" r="10" strokeWidth={2} />
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4" />
          </svg>
        );
      case 'failed':
        return (
          <svg className="status-icon failed" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <circle cx="12" cy="12" r="10" strokeWidth={2} />
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 9l-6 6M9 9l6 6" />
          </svg>
        );
      default:
        return null;
    }
  };

  const getTypeIcon = (type) => {
    switch (type) {
      case 'video':
        return '🎥';
      case 'audio':
        return '🎵';
      case 'instrumental':
        return '🎹';
      default:
        return '📁';
    }
  };

  const getStatusBadgeClass = (status) => {
    switch (status) {
      case 'completed':
        return 'badge-success';
      case 'downloading':
      case 'processing':
        return 'badge-info';
      case 'failed':
        return 'badge-error';
      default:
        return 'badge-warning';
    }
  };

  const formatDate = (dateString) => {
    const date = new Date(dateString);
    return new Intl.DateTimeFormat('fr-FR', {
      day: '2-digit',
      month: 'short',
      hour: '2-digit',
      minute: '2-digit',
    }).format(date);
  };

  const truncateUrl = (url, maxLength = 50) => {
    if (url.length <= maxLength) return url;
    return url.substring(0, maxLength) + '...';
  };

  return (
    <motion.div
      className="download-card"
      initial={{ opacity: 0, scale: 0.9 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.9 }}
      transition={{ duration: 0.3 }}
      layout
    >
      <div className="card-header">
        <div className="card-type">
          <span className="type-icon">{getTypeIcon(download.download_type)}</span>
          <span className="type-label">{download.download_type}</span>
        </div>
        <div className="card-actions">
          <span className={`badge ${getStatusBadgeClass(download.status)}`}>
            {getStatusIcon(download.status)}
            {download.status}
          </span>
          <button
            className="btn-icon btn-delete"
            onClick={() => onDelete(download.id)}
            title="Supprimer"
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
          </button>
        </div>
      </div>

      <div className="card-body">
        <div className="download-url" title={download.url}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
          </svg>
          <span>{truncateUrl(download.url)}</span>
        </div>

        <div className="download-message">
          {download.message}
        </div>

        {(download.status === 'downloading' || download.status === 'processing') && (
          <div className="progress-container">
            <div className="progress">
              <motion.div
                className="progress-bar"
                initial={{ width: 0 }}
                animate={{ width: `${download.progress}%` }}
                transition={{ duration: 0.3 }}
              />
            </div>
            <span className="progress-text">{Math.round(download.progress)}%</span>
          </div>
        )}
      </div>

      <div className="card-footer">
        <div className="download-time">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <circle cx="12" cy="12" r="10" strokeWidth={2} />
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6l4 2" />
          </svg>
          <span>{formatDate(download.created_at)}</span>
        </div>
        
        {download.status === 'completed' && download.file_path && (
          <div className="download-file">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            <span>{download.file_path}</span>
          </div>
        )}
      </div>
    </motion.div>
  );
};

export default DownloadCard;
