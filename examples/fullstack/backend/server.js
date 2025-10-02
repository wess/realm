const server = Bun.serve({
  port: 4001,
  fetch(req) {
    const url = new URL(req.url);
    
    if (url.pathname === '/api/hello') {
      return Response.json({
        message: 'Hello from the backend!',
        timestamp: new Date().toISOString()
      });
    }
    
    if (url.pathname === '/health') {
      return Response.json({ status: 'ok' });
    }
    
    return new Response('Not found', { status: 404 });
  },
});

console.log(`Backend running on http://localhost:${server.port}`);
