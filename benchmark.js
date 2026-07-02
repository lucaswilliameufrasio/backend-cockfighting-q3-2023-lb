import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '10s', target: 100 },
    { duration: '30s', target: 500 },
    { duration: '10s', target: 1000 },
    { duration: '30s', target: 1000 },
    { duration: '10s', target: 0 },
  ],
  thresholds: {
    http_req_duration: ['p(95)<2000', 'p(99)<5000'],
    http_req_failed: ['rate<0.05'],
  },
};

const BASE_URL = 'http://localhost:9999';

export default function () {
  const responses = http.batch([
    ['GET', `${BASE_URL}/health-check`, null, { tags: { name: 'health-check' } }],
    ['GET', `${BASE_URL}/pessoas?t=termo`, null, { tags: { name: 'search' } }],
    ['GET', `${BASE_URL}/pessoas/00000000-0000-0000-0000-000000000000`, null, { tags: { name: 'not-found' } }],
  ]);

  for (const res of responses) {
    check(res, { 'status is 200 or 404': (r) => r.status === 200 || r.status === 404 });
  }

  sleep(0.1);
}
