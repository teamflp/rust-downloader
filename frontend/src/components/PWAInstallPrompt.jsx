import { motion, AnimatePresence } from 'framer-motion';
import { Download, X } from 'lucide-react';
import './PWAInstallPrompt.css';

const PWAInstallPrompt = ({ show, onInstall, onDismiss }) => {
  return (
    <AnimatePresence>
      {show && (
        <motion.div
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: -20 }}
          className="pwa-install-prompt"
        >
          <div className="pwa-install-prompt-content">
            <div className="pwa-install-prompt-icon">
              <Download size={24} />
            </div>
            <div className="pwa-install-prompt-text">
              <h3>Installer l'application</h3>
              <p>Installez Rust Media Downloader pour une meilleure expérience</p>
            </div>
            <div className="pwa-install-prompt-actions">
              <button
                className="btn-install"
                onClick={onInstall}
                aria-label="Installer l'application"
              >
                Installer
              </button>
              <button
                className="btn-dismiss"
                onClick={onDismiss}
                aria-label="Ignorer"
              >
                <X size={18} />
              </button>
            </div>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
};

export default PWAInstallPrompt;

