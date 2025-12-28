/**
 * Centralized error handler to transform raw errors into user-friendly messages
 * Specifically handles HTML response errors from backend
 */
export const transformError = (err) => {
    if (!err) {
        return 'Erreur inconnue';
    }

    let errorMessage = err?.message || String(err) || 'Erreur inconnue';
    
    // ALWAYS check for HTML/JSON parsing errors first - this is the most common issue
    // Check multiple variations of the error message
    const errorString = String(errorMessage);
    const errorLower = errorString.toLowerCase();
    const fullErrorString = String(err);
    const fullErrorLower = fullErrorString.toLowerCase();
    
    // Check for HTML response errors (from transformResponse)
    // The exact error is: "Unexpected token '<', "<!DOCTYPE "... is not valid JSON"
    // We need to check for "Unexpected token" AND ("DOCTYPE" OR "<!DOCTYPE" OR "<")
    const hasUnexpectedToken = errorMessage.includes('Unexpected token') || 
                               errorString.includes('Unexpected token') ||
                               errorLower.includes('unexpected token') ||
                               fullErrorString.includes('Unexpected token') ||
                               fullErrorLower.includes('unexpected token');
    
    const hasDoctype = errorMessage.includes('DOCTYPE') ||
                      errorMessage.includes('<!DOCTYPE') ||
                      errorMessage.includes('doctype') ||
                      errorString.includes('DOCTYPE') ||
                      errorString.includes('<!DOCTYPE') ||
                      errorLower.includes('doctype') ||
                      fullErrorString.includes('DOCTYPE') ||
                      fullErrorString.includes('<!DOCTYPE') ||
                      fullErrorLower.includes('doctype');
    
    const hasHtmlTag = errorMessage.includes('<') ||
                      errorMessage.includes('<!DOCTYPE') ||
                      errorString.includes('<') ||
                      errorLower.includes('<!doctype');
    
    if (err?.isHTMLResponse ||
        errorMessage.includes('HTML_RESPONSE') ||
        (hasUnexpectedToken && (hasDoctype || hasHtmlTag)) ||
        errorMessage.includes('<!DOCTYPE') ||
        errorMessage.includes('<!doctype')) {
        return 'Le serveur backend ne répond pas correctement. Vérifiez qu\'il est démarré sur le port 9000.';
    }
    
    // Check for HTML response errors in response data
    if (err?.response?.data && typeof err.response.data === 'string' && 
        (err.response.data.startsWith('<!DOCTYPE') || 
         err.response.data.startsWith('<html') ||
         err.response.data.includes('<!DOCTYPE') ||
         err.response.data.includes('<html'))) {
        return 'Le serveur backend ne répond pas correctement. Vérifiez qu\'il est démarré sur le port 9000.';
    }
    
    // Check for connection errors
    if (err?.code === 'ECONNREFUSED' || 
        errorMessage.includes('Network Error') ||
        errorMessage.includes('ECONNREFUSED')) {
        return 'Impossible de se connecter au backend. Vérifiez qu\'il est démarré sur le port 9000.';
    }
    
    // Final check: if error message still contains the raw JSON error, replace it
    // This is a catch-all for any variation we might have missed
    if (errorMessage.includes('Unexpected token')) {
        // Check if it's related to HTML/DOCTYPE in any way
        if (errorMessage.includes('DOCTYPE') || 
            errorMessage.includes('<!DOCTYPE') ||
            errorMessage.includes('doctype') ||
            errorMessage.includes('<') ||
            errorString.includes('DOCTYPE') ||
            errorString.includes('<!DOCTYPE') ||
            errorString.includes('<') ||
            errorLower.includes('doctype') ||
            errorLower.includes('<!doctype') ||
            fullErrorString.includes('DOCTYPE') ||
            fullErrorString.includes('<!DOCTYPE') ||
            fullErrorLower.includes('doctype')) {
            return 'Le serveur backend ne répond pas correctement. Vérifiez qu\'il est démarré sur le port 9000.';
        }
    }
    
    // Return original error message if no transformation needed
    return errorMessage;
};

