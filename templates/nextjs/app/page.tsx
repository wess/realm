'use client';

import { useState, useEffect } from 'react';

export default function Home() {
  const [message, setMessage] = useState('Loading...');

  useEffect(() => {
    fetch('/api/health')
      .then(res => res.json())
      .then(data => setMessage(`API Status: ${data.status}`))
      .catch(() => setMessage('API connection failed'));
  }, []);

  return (
    <div style={{ padding: '2rem', textAlign: 'center' }}>
      <h1>Next.js Full-Stack App</h1>
      <p>{message}</p>
    </div>
  );
}
