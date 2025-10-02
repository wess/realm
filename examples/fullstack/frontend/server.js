const server = Bun.serve({
  port: 4000,
  fetch(req) {
    return new Response(Bun.file("index.html"));
  },
});

console.log(`Frontend running on http://localhost:${server.port}`);
