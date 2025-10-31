<template>
  <div id="app">
    <h1>Realm Vue + Express</h1>
    <p>{{ message }}</p>
    <button @click="fetchData">Fetch Backend Data</button>
    <ul v-if="users.length">
      <li v-for="user in users" :key="user.id">{{ user.name }}</li>
    </ul>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'

const message = ref('Loading...')
const users = ref([])

const fetchData = async () => {
  try {
    const response = await fetch('/api/users')
    users.value = await response.json()
  } catch (error) {
    console.error('Failed to fetch users:', error)
  }
}

onMounted(async () => {
  try {
    const response = await fetch('/api/health')
    const data = await response.json()
    message.value = `Connected to backend! Status: ${data.status}`
  } catch (error) {
    message.value = 'Failed to connect to backend'
  }
})
</script>

<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  text-align: center;
  color: #2c3e50;
  margin-top: 60px;
}

h1 {
  color: #42b883;
}

button {
  background-color: #42b883;
  color: white;
  border: none;
  padding: 10px 20px;
  border-radius: 5px;
  cursor: pointer;
  margin: 10px;
}
</style>
