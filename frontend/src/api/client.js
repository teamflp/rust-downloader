import axios from 'axios';

// Use relative URLs to leverage Vite proxy in development
// In production, this should be configured via environment variable
const API_BASE_URL = import.meta.env.VITE_API_URL || '';

const apiClient = axios.create({
    baseURL: API_BASE_URL,
    headers: {
        'Content-Type': 'application/json',
    },
});

export const downloadAPI = {
    // Create a new download
    createDownload: async (downloadData) => {
        const response = await apiClient.post('/api/downloads', downloadData);
        return response.data;
    },

    // Get all downloads
    getDownloads: async () => {
        const response = await apiClient.get('/api/downloads');
        return response.data;
    },

    // Get a specific download by ID
    getDownload: async (id) => {
        const response = await apiClient.get(`/api/downloads/${id}`);
        return response.data;
    },

    // Delete a download
    deleteDownload: async (id) => {
        await apiClient.delete(`/api/downloads/${id}`);
    },
};

export default apiClient;
