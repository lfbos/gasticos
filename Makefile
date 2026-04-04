.PHONY: dev dev-backend dev-frontend test test-backend test-frontend test-parsers lint fmt build migrate migrate-create db-reset db-setup clean coverage coverage-html coverage-lcov

# Development
dev:
	docker-compose up

dev-backend:
	cd backend && cargo watch -x 'run -p api'

dev-frontend:
	cd frontend && npm run dev

# Database (Diesel)
migrate:
	cd backend && diesel migration run

migrate-create:
	cd backend && diesel migration generate $(NAME)

db-reset:
	cd backend && diesel database reset

db-setup:
	cd backend && diesel setup

# Testing
test: test-backend test-frontend

test-backend:
	cd backend && cargo test --workspace

test-frontend:
	cd frontend && npm test

test-parsers:
	cd backend && cargo test -p parser

# Coverage
coverage:
	cd backend && cargo llvm-cov --workspace

coverage-html:
	cd backend && cargo llvm-cov --workspace --html --open

coverage-lcov:
	cd backend && cargo llvm-cov --workspace --lcov --output-path coverage.lcov

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
