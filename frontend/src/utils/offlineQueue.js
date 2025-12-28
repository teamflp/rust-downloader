const QUEUE_STORAGE_KEY = 'offline-download-queue';
const MAX_QUEUE_SIZE = 100;

/**
 * Offline download queue manager
 * Stores download requests when offline and processes them when back online
 */
export class OfflineQueue {
  /**
   * Get all queued downloads
   */
  static getQueue() {
    try {
      const queue = localStorage.getItem(QUEUE_STORAGE_KEY);
      return queue ? JSON.parse(queue) : [];
    } catch (error) {
      console.error('Error reading offline queue:', error);
      return [];
    }
  }

  /**
   * Add a download to the queue
   */
  static addToQueue(downloadRequest) {
    try {
      const queue = this.getQueue();
      
      // Check if already in queue (by URL)
      const exists = queue.some(item => item.url === downloadRequest.url);
      if (exists) {
        return { success: false, error: 'Download already in queue' };
      }

      // Check queue size
      if (queue.length >= MAX_QUEUE_SIZE) {
        return { success: false, error: 'Queue is full. Please wait for downloads to process.' };
      }

      // Add metadata
      const queuedItem = {
        ...downloadRequest,
        queuedAt: new Date().toISOString(),
        id: `offline-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        status: 'pending',
      };

      queue.push(queuedItem);
      localStorage.setItem(QUEUE_STORAGE_KEY, JSON.stringify(queue));

      // Dispatch event for UI updates
      window.dispatchEvent(new CustomEvent('offline-queue-updated', { 
        detail: { queue, action: 'added' } 
      }));

      return { success: true, item: queuedItem };
    } catch (error) {
      console.error('Error adding to offline queue:', error);
      return { success: false, error: error.message };
    }
  }

  /**
   * Remove a download from the queue
   */
  static removeFromQueue(itemId) {
    try {
      const queue = this.getQueue();
      const filtered = queue.filter(item => item.id !== itemId);
      localStorage.setItem(QUEUE_STORAGE_KEY, JSON.stringify(filtered));

      // Dispatch event for UI updates
      window.dispatchEvent(new CustomEvent('offline-queue-updated', { 
        detail: { queue: filtered, action: 'removed' } 
      }));

      return { success: true, queue: filtered };
    } catch (error) {
      console.error('Error removing from offline queue:', error);
      return { success: false, error: error.message };
    }
  }

  /**
   * Clear the entire queue
   */
  static clearQueue() {
    try {
      localStorage.removeItem(QUEUE_STORAGE_KEY);
      window.dispatchEvent(new CustomEvent('offline-queue-updated', { 
        detail: { queue: [], action: 'cleared' } 
      }));
      return { success: true };
    } catch (error) {
      console.error('Error clearing offline queue:', error);
      return { success: false, error: error.message };
    }
  }

  /**
   * Process the queue (called when back online)
   */
  static async processQueue(createDownloadCallback) {
    if (!navigator.onLine) {
      return { success: false, error: 'Still offline' };
    }

    const queue = this.getQueue();
    if (queue.length === 0) {
      return { success: true, processed: 0 };
    }

    const results = {
      processed: 0,
      succeeded: 0,
      failed: 0,
      errors: [],
    };

    // Process each item in the queue
    for (const item of queue) {
      try {
        // Mark as processing
        this.updateQueueItemStatus(item.id, 'processing');

        // Attempt to create download
        await createDownloadCallback(item);

        // Remove from queue on success
        this.removeFromQueue(item.id);
        results.processed++;
        results.succeeded++;

        // Small delay between requests to avoid overwhelming the server
        await new Promise(resolve => setTimeout(resolve, 500));
      } catch (error) {
        console.error('Error processing queued download:', error);
        results.failed++;
        results.errors.push({ id: item.id, url: item.url, error: error.message });

        // Keep failed items in queue but mark as failed
        this.updateQueueItemStatus(item.id, 'failed', error.message);
      }
    }

    return results;
  }

  /**
   * Update status of a queue item
   */
  static updateQueueItemStatus(itemId, status, error = null) {
    try {
      const queue = this.getQueue();
      const item = queue.find(item => item.id === itemId);
      
      if (item) {
        item.status = status;
        if (error) {
          item.error = error;
        }
        if (status === 'processing') {
          item.processedAt = new Date().toISOString();
        }

        localStorage.setItem(QUEUE_STORAGE_KEY, JSON.stringify(queue));
        window.dispatchEvent(new CustomEvent('offline-queue-updated', { 
          detail: { queue, action: 'updated' } 
        }));
      }
    } catch (error) {
      console.error('Error updating queue item status:', error);
    }
  }

  /**
   * Get queue size
   */
  static getQueueSize() {
    return this.getQueue().length;
  }
}

