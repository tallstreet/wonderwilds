// Real demo middleware with file watching disabled during durable browser journeys.
// Explicit page reload still exercises the application's saved-state boot path.
import base from '../../frontend/vite.config.ts';
export default env => {
  const config = typeof base === 'function' ? base(env) : base;
  return { ...config, server: { ...config.server, watch: null } };
};
