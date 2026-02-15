# Monitoring Setup

This project includes Prometheus and Grafana for monitoring and observability.

## Services

- **Prometheus**: Metrics collection and storage (http://localhost:9090)
- **Grafana**: Metrics visualization and dashboards (http://localhost:3001)

## Access

- **Grafana**: 
  - URL: http://localhost:3001
  - Default credentials: admin/admin
  - Prometheus datasource is pre-configured

- **Prometheus**:
  - URL: http://localhost:9090
  - Scrapes metrics from backend at `/metrics` endpoint

## Metrics Exposed

The backend exposes the following metrics at `http://localhost:3000/metrics`:

- `http_requests_total`: Total number of HTTP requests (counter)
  - Labels: method, path, status
- `http_request_duration_seconds`: HTTP request latency (histogram)
  - Labels: method, path, status

## Starting Monitoring Stack

```bash
# Start all services including monitoring
docker compose up -d

# Start only monitoring services
docker compose up -d prometheus grafana
```

## Viewing Metrics

1. Access Grafana at http://localhost:3001
2. Login with admin/admin
3. Create a new dashboard or explore metrics
4. Prometheus datasource is already configured

## Example Queries

In Grafana or Prometheus, try these queries:

```promql
# Request rate per endpoint
rate(http_requests_total[5m])

# 95th percentile latency
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))

# Error rate (5xx responses)
sum(rate(http_requests_total{status=~"5.."}[5m])) / sum(rate(http_requests_total[5m]))
```
