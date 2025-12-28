import { forwardRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { CheckCircle2, XCircle, AlertCircle, Info, X } from 'lucide-react';
import './Toast.css';

const Toast = forwardRef(({ toast, onClose }, ref) => {
  const getIcon = () => {
    switch (toast.type) {
      case 'success':
        return <CheckCircle2 size={20} />;
      case 'error':
        return <XCircle size={20} />;
      case 'warning':
        return <AlertCircle size={20} />;
      default:
        return <Info size={20} />;
    }
  };

  return (
    <motion.div
      ref={ref}
      initial={{ opacity: 0, y: -20, scale: 0.95 }}
      animate={{ opacity: 1, y: 0, scale: 1 }}
      exit={{ opacity: 0, scale: 0.95, transition: { duration: 0.2 } }}
      className={`toast toast-${toast.type}`}
    >
      <div className="toast-content">
        <div className="toast-icon">{getIcon()}</div>
        <div className="toast-message">
          <div className="toast-title">{toast.title}</div>
          {toast.message && <div className="toast-description">{toast.message}</div>}
        </div>
        <button
          className="toast-close"
          onClick={onClose}
          aria-label="Fermer la notification"
        >
          <X size={16} />
        </button>
      </div>
      {toast.duration !== 0 && (
        <motion.div
          className="toast-progress"
          initial={{ width: '100%' }}
          animate={{ width: '0%' }}
          transition={{ duration: toast.duration / 1000, ease: 'linear' }}
        />
      )}
    </motion.div>
  );
});

Toast.displayName = 'Toast';

export const ToastContainer = ({ toasts, onClose }) => {
  return (
    <div className="toast-container" aria-live="polite" aria-atomic="true">
      <AnimatePresence mode="popLayout">
        {toasts.map((toast) => (
          <Toast key={toast.id} toast={toast} onClose={() => onClose(toast.id)} />
        ))}
      </AnimatePresence>
    </div>
  );
};

