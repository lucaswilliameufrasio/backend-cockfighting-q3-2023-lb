import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  vus: 10,
  duration: '30s',
  thresholds: {
    http_req_failed: ['rate<0.01'],
  },
};

const BASE = __ENV.BASE_URL || 'http://localhost:9999';

export default function () {
  const health = http.get(`${BASE}/health-check`, { tags: { name: 'health-check' } });
  check(health, { 'health 200': (r) => r.status === 200 });

  const search = http.get(`${BASE}/pessoas?t=test`, { tags: { name: 'search' } });
  check(search, { 'search 200': (r) => r.status === 200 });

  const notFound = http.get(`${BASE}/pessoas/00000000-0000-0000-0000-000000000000`, {
    tags: { name: 'not-found' },
  });
  check(notFound, { 'not found 200 or 404': (r) => r.status === 200 || r.status === 404 });

  const anyPath = http.get(`${BASE}/qualquer-coisa`, { tags: { name: 'any-path' } });
  check(anyPath, { 'any 200': (r) => r.status === 200 });

  sleep(0.1);
}
