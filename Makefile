# ============================================================
# ft_transcendence - Root Makefile
# ============================================================

COMPOSE_MAIN       := docker-compose.yml
ENV_FILE 		   := .env
DC				   := docker compose --env-file $(ENV_FILE)
KIBANA_URL 		   := http://localhost:5601/kibana
KIBANA_EXPORT      := infra/metrics/elk/kibana/export/ft-transcendence-dashboard.ndjson
KIBANA_ENV         := kibana.env
ELASTIC_USER       := elastic
ELASTIC_PASSWORD   := $(shell sed -n 's/^ELASTIC_PASSWORD=//p' $(ENV_FILE) | tr -d "'\"")

export DOCKER_ROOT_DIR := $(shell docker info -f '{{.DockerRootDir}}' 2>/dev/null)
export DOCKER_SOCK_PATH := $(shell docker context inspect -f '{{.Endpoints.docker.Host}}' 2>/dev/null | sed 's|unix://||')
export HOSTNAME := $(shell hostname)

GREEN  := \033[0;32m
YELLOW := \033[0;33m
RED    := \033[0;31m
CYAN   := \033[0;36m
RESET  := \033[0m

define banner
	@echo ""
	@echo "$(CYAN)============================================================$(RESET)"
	@echo "$(CYAN)  $(1)$(RESET)"
	@echo "$(CYAN)============================================================$(RESET)"
	@echo ""
endef

# ============================================================
# Main targets
# ============================================================

.PHONY: all up dev down stop restart re \
        build ps logs \
        clean fclean ultraclean \
		kibana-import kibana-wait kibana-password \
        publish-assets \
		prod front-build docs-build \
        check-env check-docker-root \
        help

all: prod

up: check-env check-docker-root kibana-password
	@echo "$(GREEN)>> Starting the full stack (build + up -d)$(RESET)"
	$(DC) --env-file $(KIBANA_ENV) up -d --build && $(MAKE) publish-assets && $(MAKE) kibana-import

dev: check-env check-docker-root kibana-password
	$(call banner, DEV MODE — build the stack and attach logs)
	$(DC) --env-file $(KIBANA_ENV) up --build && $(MAKE) publish-assets && $(MAKE) kibana-import

publish-assets:
	@echo "$(GREEN)>> Publishing assets/ to MinIO$(RESET)"
	@./scripts/publish-assets.sh

stop:
	@echo "$(YELLOW)>> Stopping containers$(RESET)"
	$(DC) stop

restart:
	@echo "$(YELLOW)>> Restarting $(if $(c),$(c),all containers)$(RESET)"
	$(DC) restart $(c)

down:
	@echo "$(YELLOW)>> down (containers + networks removed, volumes kept)$(RESET)"
	$(DC) down

build: check-env check-docker-root kibana-password
	@echo "$(GREEN)>> Building without cache$(RESET)"
	$(DC) --env-file $(KIBANA_ENV) build --no-cache

re: down up

# ============================================================
# Production build (React + documentation)
# ============================================================

prod: front-build docs-build up
	$(call banner, PROD BUILD COMPLETE + STACK STARTED)
	@echo "    frontend : $(CURDIR)/front/dist"
	@echo "    docs     : $(CURDIR)/documentation/build"

front-build:
	$(call banner, PROD BUILD — REACT FRONTEND)
	@cd front && npm install && npm run build

docs-build:
	$(call banner, PROD BUILD — API DOCUMENTATION)
	@cd documentation && npm install && npm run build

ps:
	$(DC) ps

logs:
	$(DC) logs -f --tail=100

# ============================================================
# Cleanup
# ============================================================

clean: down
	@rm -f $(KIBANA_ENV)
	@echo "$(YELLOW)>> clean done (volumes and images kept)$(RESET)"

fclean: down
	@rm -f $(KIBANA_ENV)
	@echo "$(RED)>> fclean: removing project images$(RESET)"
	$(DC) down --rmi local
	@echo "$(YELLOW)>> fclean done$(RESET)"

## /!\ Destructive: wipes all persisted data
ultraclean: down
	@rm -f $(KIBANA_ENV)
	@echo "$(RED)>> ULTRACLEAN: removing images + volumes + orphan networks$(RESET)"
	$(DC) down --rmi all --volumes --remove-orphans
	@echo "$(RED)>> Cleaning up remaining dangling volumes$(RESET)"
	-docker volume prune -f
	@echo "$(RED)>> ultraclean done, everything is back to zero$(RESET)"

# ============================================================
# Kibana password
# ============================================================

kibana-password: check-env
	@echo "$(GREEN)>> Ensuring elasticsearch is up to regenerate the Kibana password$(RESET)"
	@echo "$(YELLOW)>> Note: first boot of elasticsearch can take 30-60s to become healthy$(RESET)"
	$(DC) up -d --wait --wait-timeout 300 elasticsearch
	@rm -f $(KIBANA_ENV)
	@echo "KIBANA_PASSWORD=$$(docker exec -i elasticsearch /usr/share/elasticsearch/bin/elasticsearch-reset-password -u kibana_system -s -b)" >> $(KIBANA_ENV)
	@echo "$(GREEN)Kibana password saved in $(KIBANA_ENV)$(RESET)"

# ============================================================
# Kibana saved objects
# ============================================================

kibana-wait:
	@echo "$(YELLOW)>> Waiting for Kibana to be available...$(RESET)"
	@until curl -s -o /dev/null -w "%{http_code}" $(KIBANA_URL)/api/status | grep -q "200"; do \
		printf "."; \
		sleep 2; \
	done
	@echo ""
	@echo "$(GREEN)Kibana is up$(RESET)"

kibana-import: kibana-wait
	@echo "$(GREEN)>> Importing Kibana saved objects$(RESET)"
	@curl -s -u $(ELASTIC_USER):$(ELASTIC_PASSWORD) -X POST "$(KIBANA_URL)/api/saved_objects/_import?overwrite=true" \
		-H "kbn-xsrf: true" \
		--form file=@$(KIBANA_EXPORT) \
		| grep -q '"success":true' \
		&& echo "$(GREEN)Import successful$(RESET)" \
		|| echo "$(RED)Import failed$(RESET)"
# ============================================================
# Guard rails
# ============================================================

check-env:
	@if [ ! -f $(ENV_FILE) ]; then \
		echo "$(RED)Error: $(ENV_FILE) file not found at project root.$(RESET)"; \
		exit 1; \
	fi
	@echo "$(GREEN).env is here$(RESET)"

check-docker-root:
	@if [ -z "$(DOCKER_ROOT_DIR)" ]; then \
		echo "$(RED)Error: could not determine DOCKER_ROOT_DIR (docker info failed).$(RESET)"; \
		exit 1; \
	fi
	@echo "$(GREEN)DOCKER_ROOT_DIR = $(DOCKER_ROOT_DIR)$(RESET)"

help:
	@echo "Available targets:"
	@echo "  (default)   - make prod : build React + docs + start the stack"
	@echo "  up          - build + up -d (everything, in the background)"
	@echo "  dev         - build + up with attached logs"
	@echo "  stop        - stop without removing"
	@echo "  restart     - restart existing containers"
	@echo "  down        - stop + remove containers/networks"
	@echo "  build       - rebuild without cache"
	@echo "  re          - down then up"
	@echo "  prod        - production build (React + docs) + start the stack"
	@echo "  front-build - build the React frontend into front/dist"
	@echo "  docs-build  - build the Scalar API documentation into documentation/build"
	@echo "  ps          - list containers"
	@echo "  logs        - follow logs live"
	@echo "  clean       - down (volumes/images kept)"
	@echo "  fclean      - clean + remove project images"
	@echo "  ultraclean  - fclean + remove volumes (DESTRUCTIVE)"
	@echo "  kibana-import - import saved dashboards/visualizations into Kibana"
	@echo "  kibana-password - regenerate the kibana_system password into $(KIBANA_ENV)"
