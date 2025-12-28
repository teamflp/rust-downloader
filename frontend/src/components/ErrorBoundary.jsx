import React from 'react';
import { AlertCircle } from 'lucide-react';

class ErrorBoundary extends React.Component {
  constructor(props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error) {
    return { hasError: true, error };
  }

  componentDidCatch(error, errorInfo) {
    console.error('ErrorBoundary caught an error:', error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div style={{
          padding: '2rem',
          margin: '2rem',
          background: 'var(--error-glow, rgba(239, 68, 68, 0.1))',
          border: '1px solid var(--error, #ef4444)',
          borderRadius: '8px',
          color: 'var(--error, #ef4444)',
          textAlign: 'center'
        }}>
          <AlertCircle size={48} style={{ marginBottom: '1rem' }} />
          <h2>Une erreur s'est produite</h2>
          <p>Le composant a rencontré une erreur. Veuillez recharger la page.</p>
          <button
            onClick={() => {
              this.setState({ hasError: false, error: null });
              window.location.reload();
            }}
            style={{
              marginTop: '1rem',
              padding: '0.5rem 1rem',
              background: 'var(--accent-primary, #8b5cf6)',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer'
            }}
          >
            Recharger la page
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}

export default ErrorBoundary;

