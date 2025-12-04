import { useState, useEffect, useCallback } from 'react';
import { downloadAPI } from '../api/client';

export const useDownloads = () => {
    const [downloads, setDownloads] = useState([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState(null);

    const fetchDownloads = useCallback(async () => {
        try {
            setLoading(true);
            const data = await downloadAPI.getDownloads();
            setDownloads(data);
            setError(null);
        } catch (err) {
            setError(err.message);
        } finally {
            setLoading(false);
        }
    }, []);

    const createDownload = async (downloadData) => {
        try {
            setLoading(true);
            const newDownload = await downloadAPI.createDownload(downloadData);
            setDownloads(prev => [newDownload, ...prev]);
            setError(null);

            // Immediately refresh to get the updated status (Downloading instead of Pending)
            setTimeout(async () => {
                try {
                    const updated = await downloadAPI.getDownload(newDownload.id);
                    setDownloads(prev => prev.map(d => d.id === newDownload.id ? updated : d));
                } catch (err) {
                    console.error('Failed to refresh new download:', err);
                }
            }, 500); // Wait 500ms for backend to update status

            return newDownload;
        } catch (err) {
            setError(err.message);
            throw err;
        } finally {
            setLoading(false);
        }
    };

    const deleteDownload = async (id) => {
        try {
            await downloadAPI.deleteDownload(id);
            setDownloads(prev => prev.filter(d => d.id !== id));
            setError(null);
        } catch (err) {
            setError(err.message);
            throw err;
        }
    };

    const refreshDownload = async (id) => {
        try {
            const updated = await downloadAPI.getDownload(id);
            setDownloads(prev => prev.map(d => d.id === id ? updated : d));
        } catch (err) {
            console.error('Failed to refresh download:', err);
        }
    };

    useEffect(() => {
        fetchDownloads();
    }, [fetchDownloads]);

    useEffect(() => {
        // Poll for updates every 2 seconds
        const interval = setInterval(() => {
            downloads.forEach(download => {
                // Poll for pending, downloading, and processing statuses
                if (download.status === 'pending' || download.status === 'downloading' || download.status === 'processing') {
                    refreshDownload(download.id);
                }
            });
        }, 2000);

        return () => clearInterval(interval);
    }, [downloads]);

    return {
        downloads,
        loading,
        error,
        createDownload,
        deleteDownload,
        refreshDownloads: fetchDownloads,
    };
};
