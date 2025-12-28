import { useState, useEffect, useCallback, useRef } from 'react';
import { OfflineQueue } from '../utils/offlineQueue';

/**
 * Hook to manage offline download queue
 */
export const useOfflineQueue = (createDownloadCallback) => {
  const [queue, setQueue] = useState([]);
  const [isProcessing, setIsProcessing] = useState(false);
  const [isOnline, setIsOnline] = useState(navigator.onLine);
  const createDownloadRef = useRef(createDownloadCallback);

  // Update ref when callback changes
  useEffect(() => {
    createDownloadRef.current = createDownloadCallback;
  }, [createDownloadCallback]);

  // Load queue on mount
  useEffect(() => {
    setQueue(OfflineQueue.getQueue());
  }, []);

  // Listen for queue updates
  useEffect(() => {
    const handleQueueUpdate = (event) => {
      setQueue(event.detail.queue);
    };

    window.addEventListener('offline-queue-updated', handleQueueUpdate);
    return () => {
      window.removeEventListener('offline-queue-updated', handleQueueUpdate);
    };
  }, []);

  // Add item to queue
  const addToQueue = (downloadRequest) => {
    const result = OfflineQueue.addToQueue(downloadRequest);
    if (result.success) {
      return result;
    }
    return result;
  };

  // Remove item from queue
  const removeFromQueue = (itemId) => {
    return OfflineQueue.removeFromQueue(itemId);
  };

  // Clear queue
  const clearQueue = () => {
    return OfflineQueue.clearQueue();
  };

  // Process queue
  const processQueue = useCallback(async () => {
    if (!isOnline || isProcessing || !createDownloadRef.current) {
      return { success: false, error: 'Cannot process queue' };
    }

    setIsProcessing(true);
    try {
      const results = await OfflineQueue.processQueue(createDownloadRef.current);
      return results;
    } catch (error) {
      console.error('Error processing queue:', error);
      return { success: false, error: error.message };
    } finally {
      setIsProcessing(false);
    }
  }, [isOnline, isProcessing]);

  // Monitor online status
  useEffect(() => {
    const handleOnline = () => {
      setIsOnline(true);
      // Automatically process queue when back online (use setTimeout to avoid race condition)
      setTimeout(() => {
        const currentQueue = OfflineQueue.getQueue();
        if (createDownloadRef.current && currentQueue.length > 0 && !isProcessing) {
          processQueue();
        }
      }, 1000);
    };
    const handleOffline = () => setIsOnline(false);

    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);

    return () => {
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('offline', handleOffline);
    };
  }, [isProcessing, processQueue]);

  return {
    queue,
    queueSize: queue.length,
    isProcessing,
    isOnline,
    addToQueue,
    removeFromQueue,
    clearQueue,
    processQueue,
  };
};

