import { useState, useEffect } from 'react'
import './App.css'

function App() {
  const [health, setHealth] = useState(null)
  const [message, setMessage] = useState(null)

  useEffect(() => {
    fetch('/api/health')
      .then(res => res.json())
      .then(data => setHealth(data))
      .catch(err => console.error('Failed to fetch health:', err))
  }, [])

  const fetchMessage = () => {
    fetch('/api/hello')
      .then(res => res.json())
      .then(data => setMessage(data.message))
      .catch(err => console.error('Failed to fetch message:', err))
  }

  return (
    <div className="App">
      <h1>Realm React + FastAPI</h1>
      <p>Your full-stack app is running!</p>

      {health && (
        <div className="health-status">
          <h2>Backend Status</h2>
          <pre>{JSON.stringify(health, null, 2)}</pre>
        </div>
      )}

      <button onClick={fetchMessage}>Fetch Message from API</button>
      {message && <p className="message">{message}</p>}
    </div>
  )
}

export default App
