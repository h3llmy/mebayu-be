# Mebayu Backend

Mebayu Backend is a robust and scalable API built with Rust, leveraging the Axum web framework, SQLx for PostgreSQL interactions, and Redis for caching and rate limiting.

---

## 🚀 Getting Started

### Prerequisites

Ensure you have the following installed:

* [Rust](https://www.rust-lang.org/tools/install) (2024 edition recommended)
* [PostgreSQL](https://www.postgresql.org/download/)
* [Redis](https://redis.io/download/)
* [cargo-watch](https://github.com/watchexec/cargo-watch) (for hot reloading)
* Docker & Docker Compose (optional but recommended)

---

## ⚙️ Setup

### 1️⃣ Clone the Repository

```bash
git clone <repository-url>
cd mebayu_be
```

---

### 2️⃣ Configure Environment Variables

Copy the example environment file:

```bash
cp .env.example .env
```

Update values inside `.env`:

```env
APP_PORT=3009

DATABASE_URL=postgres://postgres:password@localhost:5432/mebayu_db
REDIS_URL=redis://127.0.0.1:6379

RUST_LOG=info
```

Make sure:

* PostgreSQL is running
* Redis is running
* Credentials match your local setup

---

## 🗄 Database Migrations

This project uses **SQLx** for database interaction and migrations.

### Install SQLx CLI

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

Verify installation:

```bash
sqlx --version
```

---

### Create Database

```bash
sqlx database create
```

---

### Create New Migration

```bash
sqlx migrate add create_users_table
```

This creates files inside:

```
migrations/
```

Example structure:

```
migrations/
 ├── 20260215120000_create_users_table.up.sql
 └── 20260215120000_create_users_table.down.sql
```

---

### Run Migrations

```bash
sqlx migrate run
```

Revert last migration:

```bash
sqlx migrate revert
```

Check migration status:

```bash
sqlx migrate info
```

---

### Automatic Migration on Startup

The application automatically runs pending migrations on startup.

Just run:

```bash
cargo run
```

---

### Reset Database (Development Only ⚠️)

```bash
sqlx database drop
sqlx database create
sqlx migrate run
```

Or with Docker:

```bash
docker-compose down -v
docker-compose up -d
```

---

### SQLx Offline Mode (Recommended for CI/CD)

Enable offline mode:

```env
SQLX_OFFLINE=true
```

Generate metadata:

```bash
cargo sqlx prepare -- --all-targets
```

Commit the generated:

```
.sqlx/
```

This allows:

* Compile-time query validation
* No database required during CI
* Safer deployments

---

## 🐳 Docker Setup (Optional)
### Production Build

Start PostgreSQL, Redis and Mebayu Backend:

```bash
docker compose up -d
```

Stop services:

```bash
docker compose down
```

Rebuild containers:

```bash
docker compose up -d --build
```

### Development Build

Start PostgreSQL, Redis and Mebayu Backend:

this will start the app with hot reload

```bash
docker compose -f docker-compose.yml up -d
```

Stop services:

```bash
docker compose -f docker-compose.yml down
```

Rebuild containers:

```bash
docker compose -f docker-compose.yml up -d --build
```

---

## 🛠 Running the Project (Local)

### Development (Hot Reload)

```bash
cargo watch -x run
```

Server will be available at:

```
http://127.0.0.1:3009
```

---

### Standard Run

```bash
cargo run
```

---

### Production Build

```bash
cargo build --release
./target/release/mebayu_be
```

For production:

```bash
RUST_LOG=warn ./target/release/mebayu_be
```

---

## 🏗 Project Structure

```
src/
├── app.rs                 # Application entry point
├── core/                  # Config, security, shared core logic
├── domain/                # Business logic & DTOs
├── infrastructure/        # DB, Redis, repositories
├── presentation/          # Controllers & routes
└── shared/                # Common utilities
```

---

## 🧪 Testing & Quality

Run tests:

```bash
cargo test
```

Lint:

```bash
cargo clippy
```

Format:

```bash
cargo fmt
```

Check formatting:

```bash
cargo fmt -- --check
```

---

## 🔐 Environment Variables

| Variable     | Description                  |
| ------------ | ---------------------------- |
| APP_PORT     | Application port             |
| DATABASE_URL | PostgreSQL connection string |
| REDIS_URL    | Redis connection string      |
| RUST_LOG     | Logging level                |

---

## 🔗 Technologies Used

* **Framework:** Axum
* **Async Runtime:** Tokio
* **Database Toolkit:** SQLx
* **Cache:** Redis
* **Validation:** Validator
* **Serialization:** Serde

---

## 📦 Deployment Notes

For production:

* Use environment-based configuration
* Use connection pooling limits
* Enable SQLx offline mode
* Run migrations before starting the service (if auto-migration disabled)
* Use reverse proxy (Nginx or Caddy)
* Monitor logs and health endpoints

---

## 🧠 Best Practices

* Do not modify old migrations after production release
* Keep migrations small and atomic
* Use UUID primary keys
* Validate input at DTO level
* Use structured logging
* Avoid blocking operations inside async handlers