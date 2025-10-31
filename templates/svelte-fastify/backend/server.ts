import Fastify from 'fastify';
import cors from '@fastify/cors';

const fastify = Fastify({ logger: true });

await fastify.register(cors, {
  origin: true
});

fastify.get('/api/health', async (request, reply) => {
  return {
    status: 'ok',
    timestamp: new Date().toISOString(),
    service: 'fastify-backend'
  };
});

fastify.get('/api/users', async (request, reply) => {
  return [
    { id: 1, name: 'Alice' },
    { id: 2, name: 'Bob' }
  ];
});

const start = async () => {
  try {
    await fastify.listen({ port: 4001 });
  } catch (err) {
    fastify.log.error(err);
    process.exit(1);
  }
};

start();
