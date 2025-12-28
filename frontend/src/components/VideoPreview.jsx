import { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { Clock, User, Eye, Loader2, AlertCircle, Image as ImageIcon, Play, Calendar } from 'lucide-react';
import { transformError } from '../utils/errorHandler';
import { useErrorState } from '../hooks/useErrorState';
import './VideoPreview.css';

const VideoPreview = ({ url, cookiesBrowser, onLoad }) => {
  const [videoInfo, setVideoInfo] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useErrorState(null);

  useEffect(() => {
    if (!url || url.trim() === '') {
      setVideoInfo(null);
      setError(null);
      return;
    }

    const fetchVideoInfo = async () => {
      setLoading(true);
      setError(null);
      try {
        const params = new URLSearchParams({ url });
        if (cookiesBrowser) {
          params.append('cookies_browser', cookiesBrowser);
        }
        
        const response = await fetch(`/api/video/info?${params.toString()}`);
        
        // Check Content-Type header first
        const contentType = response.headers.get('content-type') || '';
        if (!contentType.includes('application/json')) {
          throw new Error('Le serveur backend ne répond pas correctement. Vérifiez qu\'il est démarré sur le port 9000.');
        }
        
        // Get response text first to check if it's HTML
        const responseText = await response.text();
        
        // Check if response is HTML (error page) instead of JSON - do this BEFORE parsing
        // This MUST happen before JSON.parse to prevent the "Unexpected token" error
        const trimmed = responseText.trim();
        
        // Very thorough HTML detection - check multiple patterns
        if (trimmed.startsWith('<!DOCTYPE') || 
            trimmed.startsWith('<!doctype') ||
            trimmed.startsWith('<html') || 
            trimmed.startsWith('<HTML') ||
            trimmed.startsWith('<?xml') ||
            trimmed.includes('<!DOCTYPE') ||
            trimmed.includes('<!doctype') ||
            trimmed.includes('<html') ||
            trimmed.includes('<HTML') ||
            trimmed.includes('<body') ||
            trimmed.includes('<head') ||
            (trimmed.startsWith('<') && trimmed.length > 10 && (trimmed.includes('html') || trimmed.includes('DOCTYPE') || trimmed.includes('doctype')))) {
          throw new Error('Le serveur backend ne répond pas correctement. Vérifiez qu\'il est démarré sur le port 9000.');
        }
        
        // Check if response looks like JSON before parsing
        if (trimmed.length > 0 && !trimmed.startsWith('{') && !trimmed.startsWith('[')) {
          // Double-check: if it contains HTML tags, it's definitely HTML
          if (trimmed.includes('<') && (trimmed.includes('html') || trimmed.includes('body') || trimmed.includes('head') || trimmed.includes('DOCTYPE') || trimmed.includes('doctype'))) {
            throw new Error('Le serveur backend ne répond pas correctement. Vérifiez qu\'il est démarré sur le port 9000.');
          }
          throw new Error('Réponse invalide du serveur. Vérifiez que le backend est démarré.');
        }
        
        // Try to parse as JSON
        let info;
        try {
          info = JSON.parse(responseText);
        } catch (parseError) {
          // If we get here, it means the response wasn't HTML but also wasn't valid JSON
          // This could happen if the backend returns an error message that's not JSON
          
          // ALWAYS check for HTML content first, regardless of error message
          // The error message might contain "Unexpected token '<', "<!DOCTYPE "... is not valid JSON"
          if (trimmed.includes('<!DOCTYPE') || 
              trimmed.includes('<html') || 
              trimmed.includes('<body') ||
              trimmed.includes('<head') ||
              (trimmed.startsWith('<') && trimmed.length > 10)) {
            throw new Error('Le serveur backend ne répond pas correctement. Vérifiez qu\'il est démarré sur le port 9000.');
          }
          
          // Check if parse error message contains the HTML token error (the exact error we're trying to catch)
          if (parseError.message && (
            parseError.message.includes('Unexpected token') || 
            parseError.message.includes('<!DOCTYPE') ||
            (parseError.message.includes('<') && (trimmed.includes('<!DOCTYPE') || trimmed.includes('<html')))
          )) {
            throw new Error('Le serveur backend ne répond pas correctement. Vérifiez qu\'il est démarré sur le port 9000.');
          }
          
          // If response text starts with HTML-like content, it's definitely an HTML error
          if (trimmed.startsWith('<!DOCTYPE') || trimmed.startsWith('<html')) {
            throw new Error('Le serveur backend ne répond pas correctement. Vérifiez qu\'il est démarré sur le port 9000.');
          }
          
          // Check if the trimmed text contains HTML tags
          if (trimmed.includes('<!DOCTYPE') || trimmed.includes('<html') || trimmed.includes('<body')) {
            throw new Error('Le serveur backend ne répond pas correctement. Vérifiez qu\'il est démarré sur le port 9000.');
          }
          
          // Generic parse error
          throw new Error('Réponse invalide du serveur. Vérifiez que le backend est démarré.');
        }
        
        // Check for error in JSON response
        if (!response.ok) {
          throw new Error(info.message || `Erreur serveur (${response.status})`);
        }
        setVideoInfo(info);
        if (onLoad) onLoad(info);
      } catch (err) {
        // Use transformError for consistent error handling
        let errorMessage = transformError(err);
        
        // Check if it's a YouTube bot detection error (specific to VideoPreview)
        if (errorMessage.includes('bot') || errorMessage.includes('Sign in to confirm')) {
          if (!cookiesBrowser) {
            errorMessage = 'YouTube demande une authentification. Veuillez sélectionner votre navigateur dans les options "Authentification (Cookies)" ci-dessus pour utiliser vos cookies.';
          } else {
            errorMessage = 'Erreur d\'authentification YouTube. Assurez-vous d\'être connecté à YouTube dans le navigateur sélectionné.';
          }
        }
        
        setError(errorMessage);
        setVideoInfo(null);
      } finally {
        setLoading(false);
      }
    };

    // Debounce the API call
    const timer = setTimeout(() => {
      fetchVideoInfo();
    }, 1000);

    return () => clearTimeout(timer);
  }, [url, cookiesBrowser, onLoad]);

  if (!url || url.trim() === '') {
    return null;
  }

  if (loading) {
    return (
      <motion.div
        className="video-preview loading"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
      >
        <Loader2 className="animate-spin" size={24} />
        <span>Chargement des informations...</span>
      </motion.div>
    );
  }

  if (error) {
    return (
      <motion.div
        className="video-preview error"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
      >
        <AlertCircle size={20} />
        <span>{error}</span>
      </motion.div>
    );
  }

  if (!videoInfo) {
    return null;
  }

  const formatDuration = (seconds) => {
    if (!seconds) return 'N/A';
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    
    if (hours > 0) {
      return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    }
    return `${minutes}:${secs.toString().padStart(2, '0')}`;
  };

  const formatViewCount = (count) => {
    if (!count) return 'N/A';
    if (count >= 1000000) {
      return `${(count / 1000000).toFixed(1)}M`;
    }
    if (count >= 1000) {
      return `${(count / 1000).toFixed(1)}K`;
    }
    return count.toString();
  };

  return (
    <motion.div
      className="video-preview"
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3 }}
    >
      <div className="preview-content">
        {videoInfo.thumbnail && (
          <div className="preview-thumbnail-enhanced">
            <img
              src={videoInfo.thumbnail}
              alt={videoInfo.title}
              onError={(e) => {
                e.target.style.display = 'none';
                e.target.nextSibling.style.display = 'flex';
              }}
            />
            <div className="thumbnail-placeholder" style={{ display: 'none' }}>
              <ImageIcon size={48} />
            </div>
            <div className="thumbnail-overlay">
              {videoInfo.duration && (
                <span className="thumbnail-duration">{formatDuration(videoInfo.duration)}</span>
              )}
              <div className="thumbnail-play-button">
                <Play size={24} fill="white" />
              </div>
            </div>
          </div>
        )}
        
        <div className="preview-info">
          <h3 className="preview-title">{videoInfo.title}</h3>
          
          <div className="preview-meta">
            {videoInfo.uploader && (
              <div className="meta-item">
                <User size={16} />
                <span>{videoInfo.uploader}</span>
              </div>
            )}
            
            {videoInfo.duration && (
              <div className="meta-item">
                <Clock size={16} />
                <span>{formatDuration(videoInfo.duration)}</span>
              </div>
            )}
            
            {videoInfo.view_count && (
              <div className="meta-item">
                <Eye size={16} />
                <span>{formatViewCount(videoInfo.view_count)} vues</span>
              </div>
            )}

            {videoInfo.upload_date && (
              <div className="meta-item">
                <Calendar size={16} />
                <span>{new Date(videoInfo.upload_date).toLocaleDateString('fr-FR')}</span>
              </div>
            )}
          </div>

          {videoInfo.description && (
            <p className="preview-description">
              {videoInfo.description.length > 150 
                ? `${videoInfo.description.substring(0, 150)}...` 
                : videoInfo.description}
            </p>
          )}
        </div>
      </div>
    </motion.div>
  );
};

export default VideoPreview;

