.DEFAULT_GOAL := help

.PHONY: help build-backend build-frontend build-db migrate build-all up-all redeploy-db redeploy-api redeploy-web logs-db logs-api logs-web tear-down-all

help: ## Show available Make targets
	@echo "Available targets:"
	@awk 'BEGIN {FS = ":.*## "}; /^[a-zA-Z0-9_-]+:.*## / {printf "  %-14s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

build-backend: ## Build the API Docker image
	docker compose build api

build-frontend: ## Build the web Docker image
	docker compose build web

build-db: ## Start PostgreSQL service in background
	docker compose up -d db

migrate: ## Run API database migrations
	docker compose run --rm api loglens-api migrate

build-all: ## Build images and start DB, API, and web
	docker compose up -d --build db api web

up-all: ## Build and start DB, API, and web (preferred)
	docker compose up -d --build db api web

redeploy-db: ## Rebuild and redeploy PostgreSQL only
	docker compose up -d --build db

redeploy-api: ## Rebuild and redeploy API only
	docker compose up -d --build api

redeploy-web: ## Rebuild and redeploy web only
	docker compose up -d --build web

logs-db: ## Follow PostgreSQL logs
	docker compose logs -f db

logs-api: ## Follow API logs
	docker compose logs -f api

logs-web: ## Follow web logs
	docker compose logs -f web

tear-down-all: ## Stop and remove local stack
	docker compose down --remove-orphans
