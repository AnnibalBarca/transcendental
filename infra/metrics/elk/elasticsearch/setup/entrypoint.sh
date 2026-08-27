#!/bin/sh
set -e

echo "Waiting for Elasticsearch..."
until curl -s -u "elastic:${ELASTIC_PASSWORD}" http://elasticsearch:9200/_cluster/health | grep -q '"status"'; do
  sleep 5
done

echo "Creating snapshot repository..."
curl -s -X PUT "http://elasticsearch:9200/_snapshot/ft_transcendence_archive" \
  -u "elastic:${ELASTIC_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d '{
    "type": "fs",
    "settings": { "location": "/usr/share/elasticsearch/backups" }
  }'

echo "Creating SLM policy for archiving..."
curl -s -X PUT "http://elasticsearch:9200/_slm/policy/ft_transcendence_archive_policy" \
  -u "elastic:${ELASTIC_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d '{
    "schedule": "0 0 1 * * ?",
    "name": "<ft-archive-{now/d}>",
    "repository": "ft_transcendence_archive",
    "config": { "indices": ["ft_transcendence-logs-*"] },
    "retention": { "expire_after": "90d", "min_count": 5, "max_count": 30 }
  }'

echo "Archiving setup done."

echo "Creating ILM policy..."
curl -s -X PUT "http://elasticsearch:9200/_ilm/policy/ft_transcendence_logs_policy" \
  -u "elastic:${ELASTIC_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d '{
    "policy": {
      "phases": {
        "hot": {
          "min_age": "0ms",
          "actions": {}
        },
        "warm": {
          "min_age": "3d",
          "actions": {
            "forcemerge": { "max_num_segments": 1 }
          }
        },
        "delete": {
          "min_age": "30d",
          "actions": {
            "wait_for_snapshot": { "policy": "ft_transcendence_archive_policy" },
            "delete": {}
          }
        }
      }
    }
  }'

echo "Creating index template linked to ILM policy..."
curl -s -X PUT "http://elasticsearch:9200/_index_template/ft_transcendence_logs_template" \
  -u "elastic:${ELASTIC_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d '{
    "index_patterns": ["ft_transcendence-logs-*"],
    "template": {
      "settings": {
        "index.lifecycle.name": "ft_transcendence_logs_policy"
      },
      "mappings": {
        "properties": {
          "service_name": {
            "type": "text",
            "fields": { "keyword": { "type": "keyword", "ignore_above": 256 } }
          },
          "level": {
            "type": "text",
            "fields": { "keyword": { "type": "keyword", "ignore_above": 256 } }
          }
        }
      }
    }
  }'

echo "Attaching existing indices to ILM policy..."
curl -s -X PUT "http://elasticsearch:9200/ft_transcendence-logs-*/_settings" \
  -u "elastic:${ELASTIC_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d '{
    "index.lifecycle.name": "ft_transcendence_logs_policy"
  }'

echo "ILM setup done."

echo "Setting kibana_system password..."
curl -s -X POST "http://elasticsearch:9200/_security/user/kibana_system/_password" \
  -u "elastic:${ELASTIC_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d "{\"password\":\"${KIBANA_PASSWORD}\"}"

echo "Creating logstash_writer role..."
curl -s -X PUT "http://elasticsearch:9200/_security/role/logstash_writer" \
  -u "elastic:${ELASTIC_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d '{
    "cluster": ["manage_index_templates", "monitor"],
    "indices": [
      {
        "names": ["ft_transcendence-logs-*"],
        "privileges": ["create_index", "write", "manage"]
      }
    ]
  }'

echo "Creating logstash_internal user..."
curl -s -X POST "http://elasticsearch:9200/_security/user/logstash_internal" \
  -u "elastic:${ELASTIC_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d "{
    \"password\": \"${LOGSTASH_PASSWORD}\",
    \"roles\": [\"logstash_writer\"],
    \"full_name\": \"Logstash Internal User\"
  }"

echo "Creating kibana_viewer role..."
curl -s -X PUT "http://elasticsearch:9200/_security/role/kibana_viewer" \
  -u "elastic:${ELASTIC_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d '{
    "indices": [
      {
        "names": ["ft_transcendence-logs-*"],
        "privileges": ["read", "view_index_metadata"]
      }
    ]
  }'

echo "Creating log_viewer user..."
curl -s -X POST "http://elasticsearch:9200/_security/user/log_viewer" \
  -u "elastic:${ELASTIC_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d "{
    \"password\": \"${VIEWER_PASSWORD}\",
    \"roles\": [\"kibana_viewer\", \"kibana_user\"],
    \"full_name\": \"Log Viewer\"
  }"

echo "Security setup done."