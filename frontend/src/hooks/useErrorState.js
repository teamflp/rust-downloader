import { useState, useCallback } from 'react';
import { transformError } from '../utils/errorHandler';

/**
 * Custom hook that automatically transforms errors before storing them
 * This ensures that "Unexpected token" errors are never stored in state
 */
export const useErrorState = (initialError = null) => {
  // Transform initial error if it exists - this is critical to prevent old errors from showing
  let transformedInitial = null;
  if (initialError) {
    const initial = typeof initialError === 'string' ? { message: initialError } : initialError;
    transformedInitial = transformError(initial);
    // Double-check: if it still contains "Unexpected token", force the safe message
    if (transformedInitial.includes('Unexpected token') && 
        (transformedInitial.includes('DOCTYPE') || transformedInitial.includes('<!DOCTYPE'))) {
      transformedInitial = 'Le serveur backend ne répond pas correctement. Vérifiez qu\'il est démarré sur le port 9000.';
    }
  }
  
  const [error, setErrorState] = useState(transformedInitial);

  const setError = useCallback((err) => {
    if (err === null || err === undefined || err === '') {
      setErrorState(null);
      return;
    }

    // Transform error before storing
    // Handle both string errors and Error objects
    let errorToTransform = err;
    if (typeof err === 'string') {
      errorToTransform = { message: err };
    }
    
    const transformedError = transformError(errorToTransform);
    
    // Double-check: if the transformed error still contains "Unexpected token", transform again
    // This ensures we never store the raw error
    if (transformedError.includes('Unexpected token') && 
        (transformedError.includes('DOCTYPE') || transformedError.includes('<!DOCTYPE'))) {
      setErrorState('Le serveur backend ne répond pas correctement. Vérifiez qu\'il est démarré sur le port 9000.');
      return;
    }
    
    setErrorState(transformedError);
  }, []);

  const clearError = useCallback(() => {
    setErrorState(null);
  }, []);

  return [error, setError, clearError];
};

