<script>
  import { onMount } from 'svelte';

  let message = 'Loading...';

  onMount(async () => {
    try {
      const response = await fetch('/api/health');
      const data = await response.json();
      message = `Connected to backend! Status: ${data.status}`;
    } catch (error) {
      message = 'Failed to connect to backend';
    }
  });
</script>

<h1>Realm SvelteKit + Fastify</h1>
<p>{message}</p>

<style>
  h1 {
    color: #ff3e00;
    font-family: 'Helvetica Neue', Arial, sans-serif;
    text-align: center;
  }

  p {
    text-align: center;
  }
</style>
