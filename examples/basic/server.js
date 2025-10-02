const server = Bun.serve({
  port: 4000,
  fetch(req) {
    return new Response("Hello from Realm! 🏰");
  },
});

console.log(`Server running on http://localhost:${server.port}`);
