# Gasticos

Colombian bank statement analyzer. Upload your bank statements and get insights into your spending patterns.

## Tech Stack

- **Backend**: Rust (Actix Web + Tokio), PostgreSQL 16, Redis, S3/MinIO
- **Frontend**: React + TypeScript + Vite + Tailwind CSS + Recharts
- **Auth**: JWT + Refresh Tokens (argon2 hashing)

## Planned Bank Support

### Traditional Banks
- Bancolombia
- Davivienda
- Banco de Bogotá
- BBVA Colombia
- Banco de Occidente
- Scotiabank Colpatria
- Itaú Colombia

### Digital Banks / Neobanks
- Nequi
- Daviplata
- Nu Colombia
- Lulo Bank
- MOVii
- RappiPay

## Getting Started

### Prerequisites

- Docker & Docker Compose
- Rust (latest stable, for local development)
- Node.js 20+ (for local development)

### Development with Docker

```bash
make dev
```

This starts all services:
- API: http://localhost:8080
- Frontend: http://localhost:5173
- PostgreSQL: localhost:5432
- Redis: localhost:6379
- MinIO Console: http://localhost:9001

### Local Development

```bash
# Backend
make dev-backend

# Frontend
make dev-frontend
```

### Running Tests

```bash
make test           # Run all tests
make test-backend   # Backend only
make test-frontend  # Frontend only
```

### Linting

```bash
make lint   # Run clippy + eslint
make fmt    # Format code
```

## Project Structure

```
gastico/
├── backend/
│   ├── crates/
│   │   ├── api/           # HTTP server
│   │   ├── worker/        # Background jobs
│   │   ├── parser/        # Bank statement parsing
│   │   ├── categorizer/   # Transaction categorization
│   │   └── shared/        # Models, DTOs, utils
│   └── migrations/
├── frontend/
│   └── src/
│       ├── api/           # API client
│       ├── components/    # UI components
│       ├── hooks/         # Custom hooks
│       ├── pages/         # Route pages
│       ├── stores/        # Zustand state
│       └── types/         # TypeScript types
└── shared/
    └── api-spec/          # OpenAPI spec
```
