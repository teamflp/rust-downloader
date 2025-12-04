import { useState } from 'react';
import { useDownloads } from './hooks/useDownloads';
import DownloadForm from './components/DownloadForm';
import DownloadList from './components/DownloadList';
import { CloudDownload, Sun, Moon, AlertCircle, Github } from 'lucide-react';
import './App.css';

function App() {
  const { downloads, createDownload, deleteDownload, error } = useDownloads();
  const [theme, setTheme] = useState('dark');

  const toggleTheme = () => {
    const newTheme = theme === 'dark' ? 'light' : 'dark';
    setTheme(newTheme);
    document.documentElement.setAttribute('data-theme', newTheme);
    
    // Add animation class
    const button = document.querySelector('.theme-toggle');
    button?.classList.add('changing');
    setTimeout(() => button?.classList.remove('changing'), 600);
  };

  const handleDownload = async (downloadData) => {
    await createDownload(downloadData);
  };

  const handleDelete = async (id) => {
    await deleteDownload(id);
  };

  return (
    <div className="app">
      <header className="app-header">
        <div className="container">
          <div className="header-content">
            <div className="logo">
              <div className="logo-icon">
                <CloudDownload size={32} />
              </div>
              <div className="logo-text">
                <h1>Rust Media Downloader</h1>
                <p className="tagline">Téléchargez vos médias préférés en un clic</p>
              </div>
            </div>
            
            <button
              className="btn-icon theme-toggle"
              onClick={toggleTheme}
              title={`Passer au thème ${theme === 'dark' ? 'clair' : 'sombre'}`}
            >
              {theme === 'dark' ? <Sun size={20} /> : <Moon size={20} />}
            </button>
          </div>
        </div>
      </header>

      <main className="app-main">
        <div className="container">
          {error && (
            <div className="error-banner">
              <AlertCircle size={20} />
              {error}
            </div>
          )}

          <DownloadForm onSubmit={handleDownload} />
          <DownloadList downloads={downloads} onDelete={handleDelete} />
        </div>
      </main>

      <footer className="app-footer">
        <div className="container">
          <div className="footer-content">
            <p>
              Créé avec ❤️ en <strong>Rust</strong> et <strong>React</strong>
            </p>
            <div className="footer-links">
              <a href="https://github.com/teamflp/rust-downloader" target="_blank" rel="noopener noreferrer">
                <Github size={20} />
                GitHub
              </a>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
}

export default App;
