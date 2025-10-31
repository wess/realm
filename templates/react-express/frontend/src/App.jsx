import { useState, useEffect } from 'react'
import './App.css'

function App() {
  const [health, setHealth] = useState(null)

  useEffect(() => {
    fetch('/api/health')
      .then(res => res.json())
      .then(data => setHealth(data))
      .catch(err => console.error('Failed to fetch health:', err))
  }, [])

  return (
    <div className="App">
      <h1>Realm React + Express</h1>
      <p>Your full-stack app is running!</p>

      {health && (
        <div className="health-status">
          <h2>Backend Status</h2>
          <pre>{JSON.stringify(health, null, 2)}</pre>
        </div>
      )}
    </div>
  )
}

export default App
