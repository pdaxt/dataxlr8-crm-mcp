# :handshake: dataxlr8-crm-mcp

Full-lifecycle CRM for AI agents — contacts, deals, pipeline, tasks, and activity tracking over MCP.

[![Rust](https://img.shields.io/badge/Rust-2024_edition-orange?logo=rust)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-rmcp_0.17-blue)](https://modelcontextprotocol.io/)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

## What It Does

Gives AI agents a complete CRM through MCP tool calls. Create and search contacts, manage deals through pipeline stages, log every call/email/meeting, assign ownership, tag for segmentation, and bulk import/export — all persisted in PostgreSQL with schema isolation.

## Architecture

```
                    ┌─────────────────────────┐
AI Agent ──stdio──▶ │  dataxlr8-crm-mcp       │
                    │  (rmcp 0.17 server)      │
                    └──────────┬──────────────┘
                               │ sqlx 0.8
                               ▼
                    ┌─────────────────────────┐
                    │  PostgreSQL              │
                    │  schema: crm             │
                    │  ├── contacts            │
                    │  ├── deals               │
                    │  ├── activities           │
                    │  ├── tasks               │
                    │  ├── contact_tags         │
                    │  └── contact_interactions │
                    └─────────────────────────┘
```

## Tools

| Tool | Description |
|------|-------------|
| `create_contact` | Create a new contact with name, email, company, phone |
| `search_contacts` | Search contacts by name, email, or company |
| `upsert_deal` | Create or update a deal with value and stage |
| `move_deal` | Move a deal to a different pipeline stage |
| `log_activity` | Log a call, email, or meeting against a contact |
| `get_pipeline` | View the full deal pipeline with stage breakdown |
| `assign_contact` | Assign a contact to an owner |
| `create_task` | Create a follow-up task with due date |
| `import_contacts` | Bulk import contacts from a JSON array |
| `export_contacts` | Export all contacts as JSON |
| `add_interaction` | Record a contact interaction with notes |
| `tag_contact` | Add tags to a contact for segmentation |

## Quick Start

```bash
git clone https://github.com/pdaxt/dataxlr8-crm-mcp
cd dataxlr8-crm-mcp
cargo build --release

export DATABASE_URL=postgres://user:pass@localhost:5432/dataxlr8
./target/release/dataxlr8-crm-mcp
```

The server auto-creates the `crm` schema and all tables on first run.

## Configuration

| Variable | Required | Description |
|----------|----------|-------------|
| `DATABASE_URL` | Yes | PostgreSQL connection string |
| `LOG_LEVEL` | No | Tracing level (default: `info`) |

## Claude Desktop Integration

Add to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "dataxlr8-crm": {
      "command": "./target/release/dataxlr8-crm-mcp",
      "env": {
        "DATABASE_URL": "postgres://user:pass@localhost:5432/dataxlr8"
      }
    }
  }
}
```

## Part of DataXLR8

One of 14 Rust MCP servers that form the [DataXLR8](https://github.com/pdaxt) platform — a modular, AI-native business operations suite. Each server owns a single domain, shares a PostgreSQL instance, and communicates over the Model Context Protocol.

## License

MIT
