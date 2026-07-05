# backend-cockfighting-q3-2023-lb

Load Balancer em Rust (Pingora 0.8.1) para a Rinha de Backend 2023 Q3.

Stack: `pingora` (Cloudflare) + `tokio`

---

## Quick start

```bash
# Teste com mock upstreams
docker compose -f docker-compose.test.yml up -d

# LB escuta em :9999, distribui entre api1 (mock) e api2 (mock)
curl http://localhost:9999/health-check
curl http://localhost:9999/pessoas
curl http://localhost:9999/qualquer-coisa

# Parar
docker compose -f docker-compose.test.yml down -v
```

## Uso com APIs reais

O LB é usado como imagem nos benchmarks de Go e Rust:

```yaml
# docker-compose.benchmark.yml
lb:
  image: ghcr.io/lucaswilliameufrasio/backend-cockfighting-q3-2023-lb:<SHA>
  environment:
    LISTEN: "0.0.0.0:9999"
    UPSTREAMS: "api1:8080,api2:8081"
```

### Variáveis de ambiente do LB

| Variável | Padrão | Descrição |
|---|---|---|
| `LISTEN` | `0.0.0.0:9999` | Endereço onde o LB escuta |
| `UPSTREAMS` | `api1:8080,api2:8081` | Lista de upstreams separada por vírgula |
| `HEALTH_CHECK_INTERVAL` | `5` | Intervalo em segundos entre health checks TCP |

## Configuração interna (código)

- **Algoritmo**: Round-robin
- **Timeouts**:
  - `connection_timeout`: 5s
  - `read_timeout`: 10s
  - `idle_timeout`: 60s
- **Health check**: TCP (conexão), a cada 5s
- **Keep-alive**: Header `Connection: keep-alive` enviado para upstreams
- **Worker threads**: automático (tokio multi-thread, 1 por CPU)
- **Listen backlog**: padrão Pingora (~1024)

## Benchmark do LB isolado

O repositório tem um benchmark com k6 em `benchmark.js`:

```bash
# Instalar k6 (Ubuntu/Debian)
sudo apt-get install -y gnupg
curl -fsSL https://dl.k6.io/key.gpg | sudo gpg --dearmor -o /usr/share/keyrings/k6.gpg
echo "deb [signed-by=/usr/share/keyrings/k6.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update && sudo apt-get install -y k6

# Subir stack de teste
docker compose -f docker-compose.test.yml up -d

# Rodar benchmark
k6 run benchmark.js

# O benchmark espera:
# - P95 < 2s
# - P99 < 5s
# - failure rate < 5%

# Parar
docker compose -f docker-compose.test.yml down -v
```

## Troca de SHA da imagem

Quando houver mudanças no LB, atualizar a SHA nos repositórios consumer:

```bash
# No repositório Go ou Rust
git checkout -b chore/update-lb
# Editar docker-compose.benchmark.yml
# Substituir a SHA do LB pela nova
git add -A && git commit -m "chore: bump LB to <SHA>"
git push origin HEAD
```

## Troubleshooting

| Sintoma | Causa |
|---|---|
| `no live upstreams` no log | Health check TCP falhou; upstream pode estar fora |
| Timeout excessivo | `read_timeout` ou `connection_timeout` baixo demais |
| Conexões não reusadas | Falta header `Connection: keep-alive` |
| LB para de responder após stress | Verificar se o PID ainda existe; default sem restart |
