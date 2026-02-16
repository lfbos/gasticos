.PHONY: dev dev-backend dev-frontend test test-backend test-frontend test-parsers lint fmt build migrate migrate-create db-reset clean

# Development
dev:
	docker-compose up

dev-backend:
	cd backend && cargo watch -x 'run -p api'

dev-frontend:
	cd frontend && npm run dev

# Database
migrate:
	cd backend && sqlx migrate run

migrate-create:
	cd backend && sqlx migrate add $(NAME)

db-reset:
	cd backend && sqlx database drop -y && sqlx database create && sqlx migrate run

# Testing
test: test-backend test-frontend

test-backend:
	cd backend && cargo test --workspace

test-frontend:
	cd frontend && npm test

test-parsers:
	cd backend && cargo test -p parser

# Linting
lint:
	cd backend && cargo clippy --workspace -- -D warnings
	cd frontend && npm run lint

fmt:
	cd backend && cargo fmt --all
	cd frontend && npm run format

# Build
build:
	cd backend && cargo build --release
	cd frontend && npm run build

# Clean
clean:
	cd backend && cargo clean
	cd frontend && rm -rf dist node_modules

# Docker helpers
docker-build:
	docker-compose build

docker-down:
	docker-compose down

docker-logs:
	docker-compose logs -f

docker-clean:
	docker-compose down -v --rmi local
