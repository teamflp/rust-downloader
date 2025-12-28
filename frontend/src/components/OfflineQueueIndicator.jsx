import { motion } from 'framer-motion';
import { CloudOff, Clock, AlertCircle } from 'lucide-react';
import './OfflineQueueIndicator.css';

const OfflineQueueIndicator = ({ queueSize, isProcessing, isOnline }) => {
  if (isOnline && queueSize === 0) {
    return null;
  }

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      className="offline-queue-indicator"
    >
      {!isOnline ? (
        <div className="offline-status">
          <CloudOff size={18} />
          <span>Mode hors-ligne</span>
          {queueSize > 0 && (
            <span className="queue-count">{queueSize} en attente</span>
          )}
        </div>
      ) : queueSize > 0 ? (
        <div className="queue-status">
          {isProcessing ? (
            <>
              <Clock size={18} className="spinning" />
              <span>Traitement de la file d'attente...</span>
            </>
          ) : (
            <>
              <AlertCircle size={18} />
              <span>{queueSize} téléchargement(s) en attente</span>
            </>
          )}
        </div>
      ) : null}
    </motion.div>
  );
};

export default OfflineQueueIndicator;

