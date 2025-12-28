import { useState } from 'react';
import ReactPlayer from 'react-player';
import { X, Play, Pause, Volume2, Maximize2 } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import './MediaPlayer.css';

const MediaPlayer = ({ downloadId, fileUrl, mediaType, title, onClose }) => {
  const [playing, setPlaying] = useState(false);
  const [volume, setVolume] = useState(0.8);
  const [played, setPlayed] = useState(0);
  const [duration, setDuration] = useState(0);
  const [fullscreen, setFullscreen] = useState(false);

  const handlePlayPause = () => {
    setPlaying(!playing);
  };

  const handleVolumeChange = (e) => {
    setVolume(parseFloat(e.target.value));
  };

  const handleProgress = (state) => {
    setPlayed(state.played);
  };

  const handleDuration = (duration) => {
    setDuration(duration);
  };

  const formatTime = (seconds) => {
    if (!seconds) return '0:00';
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const isVideo = mediaType === 'video' || fileUrl.match(/\.(mp4|webm|mkv|avi|mov)$/i);
  const isAudio = mediaType === 'audio' || fileUrl.match(/\.(mp3|wav|m4a|flac|ogg|opus)$/i);

  return (
    <AnimatePresence>
      <motion.div
        className="media-player-overlay"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        onClick={onClose}
      >
        <motion.div
          className={`media-player-container ${fullscreen ? 'fullscreen' : ''}`}
          initial={{ scale: 0.9, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          exit={{ scale: 0.9, opacity: 0 }}
          onClick={(e) => e.stopPropagation()}
        >
          <div className="media-player-header">
            <h3 className="media-player-title">{title || 'Lecteur Média'}</h3>
            <button className="media-player-close" onClick={onClose} title="Fermer">
              <X size={20} />
            </button>
          </div>

          <div className="media-player-content">
            <div className="react-player-wrapper">
              <ReactPlayer
                url={fileUrl}
                playing={playing}
                volume={volume}
                controls={false}
                width="100%"
                height={isVideo ? "100%" : "80px"}
                onProgress={handleProgress}
                onDuration={handleDuration}
                onEnded={() => setPlaying(false)}
                config={{
                  file: {
                    attributes: {
                      controlsList: 'nodownload',
                    },
                  },
                }}
              />
            </div>

            {isAudio && (
              <div className="audio-visualizer">
                <div className="audio-wave">
                  {[...Array(20)].map((_, i) => (
                    <div
                      key={i}
                      className="wave-bar"
                      style={{
                        height: playing ? `${Math.random() * 60 + 20}%` : '20%',
                        animationDelay: `${i * 0.05}s`,
                      }}
                    />
                  ))}
                </div>
              </div>
            )}
          </div>

          <div className="media-player-controls">
            <button
              className="control-button play-pause"
              onClick={handlePlayPause}
              title={playing ? 'Pause' : 'Lecture'}
            >
              {playing ? <Pause size={24} /> : <Play size={24} />}
            </button>

            <div className="progress-container">
              <div className="progress-time">{formatTime(played * duration)}</div>
              <input
                type="range"
                min={0}
                max={1}
                step="any"
                value={played}
                onChange={(e) => {
                  // Seek is handled by ReactPlayer internally via onSeek
                  // For now, we'll just update played state
                  setPlayed(parseFloat(e.target.value));
                }}
                className="progress-bar"
              />
              <div className="progress-time">{formatTime(duration)}</div>
            </div>

            <div className="volume-container">
              <Volume2 size={20} />
              <input
                type="range"
                min={0}
                max={1}
                step="any"
                value={volume}
                onChange={handleVolumeChange}
                className="volume-bar"
              />
              <span className="volume-value">{Math.round(volume * 100)}%</span>
            </div>

            <button
              className="control-button fullscreen"
              onClick={() => setFullscreen(!fullscreen)}
              title="Plein écran"
            >
              <Maximize2 size={20} />
            </button>
          </div>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
};

export default MediaPlayer;

