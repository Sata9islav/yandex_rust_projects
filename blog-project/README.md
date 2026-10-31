# Rust Blog

A small blog application built as a Rust Cargo workspace. Users can register, sign in, and manage their own posts. Anyone can read published posts. The backend exposes both HTTP and gRPC APIs; the browser frontend uses HTTP.

## Architecture

| Crate | Purpose |
| --- | --- |
| `blog-server` | Actix Web HTTP API, Tonic gRPC API, PostgreSQL/SQLx, JWT authentication and author checks |
| `blog-proto` | Shared protobuf definitions and generated gRPC types |
| `blog-client` | Reusable HTTP (`reqwest`) and gRPC (`tonic`) client library |
| `blog-cli` | Command-line interface built on `blog-client` |
| `blog-wasm` | Browser frontend compiled to WebAssembly, calling the HTTP API directly |

```text
CLI ──> blog-client ── HTTP / gRPC ──> blog-server ──> PostgreSQL
                                  ^
Browser ──> blog-wasm ── HTTP ────┘
                   blog-proto: shared gRPC contract
```

## Prerequisites

- Rust toolchain and Cargo
- PostgreSQL (running locally or accessible through `DATABASE_URL`)
- `wasm-pack` and the `wasm32-unknown-unknown` Rust target for the frontend
- Python 3 for serving the static frontend

## Database and environment

1. Create a PostgreSQL database (adjust the database user/password for your setup):

   ```bash
   createdb blog_db
   ```

2. Create `blog-server/.env`:

   ```dotenv
   DATABASE_URL=postgres://postgres:YOUR_PASSWORD@localhost:5432/blog_db
   JWT_SECRET=REPLACE_WITH_A_RANDOM_SECRET
   ```

   Generate a random JWT secret on Linux/macOS:

   ```bash
   openssl rand -hex 32
   ```

   Copy the generated value into `JWT_SECRET`. **Do not commit `.env` or JWT tokens.**

The server loads `blog-server/.env` and applies SQLx migrations automatically on startup.

## Build and run

Run the following commands from the workspace root unless noted otherwise.

**Backend (HTTP: `127.0.0.1:8080`, gRPC: `127.0.0.1:50051`):**

```bash
cargo build --workspace
cargo run -p blog-server
```

**Client library:** It is a library, not a separate executable. Build or test it with:

```bash
cargo build -p blog-client
cargo test -p blog-client
```

The integration tests require the backend and PostgreSQL to be running.

**CLI (in another terminal):**

```bash
cargo run -p blog-cli -- --help
cargo run -p blog-cli -- list --limit 20 --offset 0
cargo run -p blog-cli -- --grpc list --limit 10
```

**WASM frontend:**

```bash
rustup target add wasm32-unknown-unknown
# Install if needed: cargo install wasm-pack --locked
cd blog-wasm
wasm-pack build --target web
python3 -m http.server 8000
```

Open **http://localhost:8000**. Keep the backend running on port `8080`. The frontend uses HTTP only and stores its JWT in browser `localStorage`.

## Try the API

### HTTP with curl

```bash
# Register (use a new username/email on each run)
curl -X POST http://127.0.0.1:8080/api/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","email":"alice@example.com","password":"secret123"}'

# Log in; copy the `token` value from the JSON response
curl -X POST http://127.0.0.1:8080/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","password":"secret123"}'

# Read public posts
curl 'http://127.0.0.1:8080/api/posts?limit=20&offset=0'

# Paste the login token locally; do not commit it
TOKEN='PASTE_JWT_HERE'

# Create a post (requires JWT)
curl -X POST http://127.0.0.1:8080/api/posts \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"title":"Hello Rust","content":"My first post"}'

# Replace 1 with the ID returned by the create request
curl http://127.0.0.1:8080/api/posts/1
curl -X PUT http://127.0.0.1:8080/api/posts/1 \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"title":"Updated title","content":"Updated content"}'
curl -X DELETE http://127.0.0.1:8080/api/posts/1 \
  -H "Authorization: Bearer $TOKEN"
```

Only the author of a post can update or delete it.

### CLI

```bash
cargo run -p blog-cli -- register --username bob --email bob@example.com --password 'secret123'
cargo run -p blog-cli -- login --username bob --password 'secret123'
cargo run -p blog-cli -- create --title 'First post' --content 'Hello!'
cargo run -p blog-cli -- list --limit 20 --offset 0
cargo run -p blog-cli -- get --id 1
cargo run -p blog-cli -- update --id 1 --title 'New title'
cargo run -p blog-cli -- delete --id 1 --grpc
```

Use the actual post ID from the create output. Add `--grpc` to any command to use gRPC, or `--server URL` to override its default address. The CLI saves JWT in `~/.blog_token` (mode `0600` on Linux). For this educational CLI, passwords passed as arguments may appear in shell history.

### Browser

1. Open **http://localhost:8000** while the backend and frontend servers are running.
2. View posts without signing in.
3. Register or sign in using the forms.
4. Create a post, then edit or delete it using the controls on your own posts.
5. Refresh the page to verify the session persists, then sign out.

## Checks

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check -p blog-wasm --target wasm32-unknown-unknown
cd blog-wasm && wasm-pack build --target web
```

Start PostgreSQL and `blog-server` before running HTTP/gRPC integration tests. Generated `target/` and `blog-wasm/pkg/` files and secrets should be excluded from Git.
