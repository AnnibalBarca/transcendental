#!/bin/sh
set -e

echo "Waiting for Kibana..."
until curl -s http://kibana:5601/kibana/api/status | grep -q '"level":"available"'; do
  sleep 5
done

echo "Creating data view..."
curl -s -X POST "http://kibana:5601/kibana/api/data_views/data_view" \
  -u "elastic:${ELASTIC_PASSWORD}" \
  -H "kbn-xsrf: true" \
  -H "Content-Type: application/json" \
  -d '{
    "data_view": {
      "title": "ft_transcendence-logs-*",
      "name": "FT Transcendence Logs",
      "timeFieldName": "@timestamp"
    }
  }'

echo "Done."
