# dataxlr8-crm-mcp

CRM MCP server for the DataXLR8 platform.

## What It Does

Full contact and deal management through MCP tools. Create and search contacts, manage deals through pipeline stages, log activities and interactions, assign ownership, tag contacts, and bulk import/export — all backed by PostgreSQL.

## Tools

| Tool | Description |
|------|-------------|
| `create_contact` | Create a new contact (name, email, company, etc.) |
| `search_contacts` | Search contacts by name, email, or company |
| `upsert_deal` | Create or update a deal |
| `move_deal` | Move a deal to a different pipeline stage |
| `log_activity` | Log a call, email, or meeting against a contact |
| `get_pipeline` | View the deal pipeline with stage breakdown |
| `assign_contact` | Assign a contact to an owner |
| `create_task` | Create a follow-up task |
| `import_contacts` | Bulk import contacts from JSON array |
| `export_contacts` | Export contacts as JSON |
| `add_interaction` | Record a contact interaction with notes |
| `tag_contact` | Add tags to a contact |

## Quick Start

```bash
export DATABASE_URL=postgres://user:pass@localhost:5432/dataxlr8

cargo build
cargo run
```

## Schema

Creates a `crm` schema with:

| Table | Purpose |
|-------|---------|
| `crm.contacts` | Contact records (name, email, company, phone, etc.) |
| `crm.deals` | Deals with stage, value, and contact linkage |
| `crm.activities` | Activity log (calls, emails, meetings) |
| `crm.tasks` | Follow-up tasks with owner and due date |
| `crm.contact_tags` | Tags associated with contacts |
| `crm.contact_interactions` | Interaction history with notes |

## Part of the [DataXLR8](https://github.com/pdaxt) Platform
