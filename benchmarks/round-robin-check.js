import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  vus: 50,
  duration: '30s',
  thresholds: {
    http_req_failed: ['rate<0.01'],
  },
};

const BASE = __ENV.BASE_URL || 'http://localhost:9999';

export default function () {
  const resp = http.get(`${BASE}/health-check`, { tags: { name: 'health-check' } });
  check(resp, { 'health 200': (r) => r.status === 200 });

  sleep(0.05);
}

// Interpretação: após o teste, verificar os logs do LB.
// 'docker compose logs lb | grep upstream' deve mostrar
// distribuição entre api1 e api2 (mock upstreams).
// Ideal: cada upstream recebe ~50% das requisições.
