import { useState, useCallback, useRef } from 'react';

export const useRateLimit = (maxRequests = 5, timeWindow = 60000) => {
  const [isLimited, setIsLimited] = useState(false);
  const requestTimesRef = useRef([]);

  const checkLimit = useCallback(() => {
    const now = Date.now();
    
    // Remove requests outside the time window
    requestTimesRef.current = requestTimesRef.current.filter(
      (time) => now - time < timeWindow
    );

    // Check if limit exceeded
    if (requestTimesRef.current.length >= maxRequests) {
      setIsLimited(true);
      const oldestRequest = requestTimesRef.current[0];
      const waitTime = timeWindow - (now - oldestRequest);

      setTimeout(() => {
        setIsLimited(false);
        requestTimesRef.current = [];
      }, waitTime);

      return false;
    }

    // Add current request
    requestTimesRef.current.push(now);
    return true;
  }, [maxRequests, timeWindow]);

  return { isLimited, checkLimit };
};

