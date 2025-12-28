import { useEffect, useState } from 'react';

export const useNotifications = () => {
  const [permission, setPermission] = useState('default');
  const [isSupported, setIsSupported] = useState(false);

  useEffect(() => {
    if ('Notification' in window) {
      setIsSupported(true);
      setPermission(Notification.permission);
    }
  }, []);

  const requestPermission = async () => {
    if (!isSupported) return false;

    if (Notification.permission === 'default') {
      const result = await Notification.requestPermission();
      setPermission(result);
      return result === 'granted';
    }

    return Notification.permission === 'granted';
  };

  const showNotification = (title, options = {}) => {
    if (!isSupported || Notification.permission !== 'granted') {
      return;
    }

    const notification = new Notification(title, {
      icon: '/icons/icon-192x192.png',
      badge: '/icons/icon-72x72.png',
      tag: 'download-notification',
      requireInteraction: false,
      ...options,
    });

    notification.onclick = () => {
      window.focus();
      notification.close();
    };

    // Auto-close after 5 seconds
    setTimeout(() => notification.close(), 5000);

    return notification;
  };

  const notifyDownloadCompleted = (title, fileName) => {
    showNotification('Téléchargement terminé', {
      body: `${title || fileName} a été téléchargé avec succès`,
      icon: '/icons/icon-192x192.png',
    });
  };

  const notifyDownloadFailed = (title, error) => {
    showNotification('Téléchargement échoué', {
      body: `${title || 'Le téléchargement'} a échoué: ${error || 'Erreur inconnue'}`,
      icon: '/icons/icon-192x192.png',
    });
  };

  return {
    isSupported,
    permission,
    requestPermission,
    showNotification,
    notifyDownloadCompleted,
    notifyDownloadFailed,
  };
};

