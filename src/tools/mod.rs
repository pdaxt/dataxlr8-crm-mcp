use dataxlr8_mcp_core::mcp::{error_result, get_f64, get_i64, get_str, get_str_array, json_result, make_schema};
use dataxlr8_mcp_core::Database;
use rmcp::model::*;
use rmcp::service::{RequestContext, RoleServer};
use rmcp::ServerHandler;
use serde::{Deserialize, Serialize};
use tracing::info;

// ============================================================================
// Data types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Contact {
    pub id: uuid::Uuid,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub company: Option<String>,
    pub title: Option<String>,
    pub phone: Option<String>,
    pub linkedin_url: Option<String>,
    pub source: Option<String>,
    pub tags: Vec<String>,
    pub custom_fields: serde_json::Value,
    pub owner_id: Option<uuid::Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Deal {
    pub id: uuid::Uuid,
    pub title: String,
    pub contact_id: Option<uuid::Uuid>,
    pub stage: String,
    pub value: Option<String>,
    pub owner_id: Option<uuid::Uuid>,
    pub expected_close: Option<chrono::NaiveDate>,
    pub notes: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Activity {
    pub id: uuid::Uuid,
    pub contact_id: Option<uuid::Uuid>,
    pub deal_id: Option<uuid::Uuid>,
    pub activity_type: String,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Task {
    pub id: uuid::Uuid,
    pub contact_id: Option<uuid::Uuid>,
    pub deal_id: Option<uuid::Uuid>,
    pub title: String,
    pub due_date: Option<chrono::NaiveDate>,
    pub completed: bool,
    pub owner_id: Option<uuid::Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ContactTag {
    pub id: uuid::Uuid,
    pub contact_id: uuid::Uuid,
    pub tag: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ContactInteraction {
    pub id: uuid::Uuid,
    pub contact_id: uuid::Uuid,
    pub interaction_type: String,
    pub subject: Option<String>,
    pub notes: Option<String>,
    pub occurred_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PipelineRow {
    pub stage: String,
    pub deal_count: i64,
    pub total_value: Option<String>,
}

fn build_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "create_contact".into(),
            title: None,
            description: Some("Create a new CRM contact".into()),
            input_schema: make_schema(
                serde_json::json!({
                    "email": { "type": "string", "description": "Contact email (unique)" },
                    "first_name": { "type": "string", "description": "First name" },
                    "last_name": { "type": "string", "description": "Last name" },
                    "company": { "type": "string", "description": "Company name" },
                    "title": { "type": "string", "description": "Job title" },
                    "phone": { "type": "string", "description": "Phone number" },
                    "linkedin_url": { "type": "string", "description": "LinkedIn profile URL" },
                    "source": { "type": "string", "description": "Lead source (e.g. website, referral, linkedin)" },
                    "tags": { "type": "array", "items": { "type": "string" }, "description": "Tags for categorization" }
                }),
                vec![],
            ),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
        Tool {
            name: "search_contacts".into(),
            title: None,
            description: Some(
                "Search contacts by name, email, or company. Supports ILIKE fuzzy matching."
                    .into(),
            ),
            input_schema: make_schema(
                serde_json::json!({
                    "query": { "type": "string", "description": "Search term (matches first_name, last_name, email, company)" },
                    "tag": { "type": "string", "description": "Filter by tag" },
                    "limit": { "type": "integer", "description": "Max results (default 50)" },
                    "offset": { "type": "integer", "description": "Offset for pagination (default 0)" }
                }),
                vec!["query"],
            ),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
        Tool {
            name: "upsert_deal".into(),
            title: None,
            description: Some(
                "Create or update a deal. If a deal with the same title+contact_id exists, it updates. Valid stages: lead, qualified, proposal, negotiation, closed_won, closed_lost"
                    .into(),
            ),
            input_schema: make_schema(
                serde_json::json!({
                    "title": { "type": "string", "description": "Deal title" },
                    "contact_id": { "type": "string", "description": "UUID of associated contact" },
                    "stage": { "type": "string", "enum": ["lead", "qualified", "proposal", "negotiation", "closed_won", "closed_lost"], "description": "Pipeline stage" },
                    "value": { "type": "number", "description": "Deal value in dollars" },
                    "owner_id": { "type": "string", "description": "UUID of deal owner" },
                    "expected_close": { "type": "string", "description": "Expected close date (YYYY-MM-DD)" },
                    "notes": { "type": "string", "description": "Deal notes" }
                }),
                vec!["title"],
            ),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
        Tool {
            name: "move_deal".into(),
            title: None,
            description: Some(
                "Move a deal to a new pipeline stage and auto-log a stage-change activity".into(),
            ),
            input_schema: make_schema(
                serde_json::json!({
                    "deal_id": { "type": "string", "description": "UUID of the deal" },
                    "new_stage": { "type": "string", "enum": ["lead", "qualified", "proposal", "negotiation", "closed_won", "closed_lost"], "description": "New pipeline stage" },
                    "notes": { "type": "string", "description": "Optional notes about the stage change" }
                }),
                vec!["deal_id", "new_stage"],
            ),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
        Tool {
            name: "log_activity".into(),
            title: None,
            description: Some(
                "Log an activity (call, email, meeting, note) against a contact or deal".into(),
            ),
            input_schema: make_schema(
                serde_json::json!({
                    "contact_id": { "type": "string", "description": "UUID of the contact" },
                    "deal_id": { "type": "string", "description": "UUID of the deal" },
                    "activity_type": { "type": "string", "enum": ["call", "email", "meeting", "note"], "description": "Type of activity" },
                    "subject": { "type": "string", "description": "Activity subject line" },
                    "body": { "type": "string", "description": "Activity details" }
                }),
                vec!["activity_type"],
            ),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
        Tool {
            name: "get_pipeline".into(),
            title: None,
            description: Some(
                "Get pipeline summary: deal count and total value per stage. Optionally filter by owner."
                    .into(),
            ),
            input_schema: make_schema(
                serde_json::json!({
                    "owner_id": { "type": "string", "description": "Filter by deal owner UUID" }
                }),
                vec![],
            ),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
        Tool {
            name: "assign_contact".into(),
            title: None,
            description: Some("Assign a contact to an owner".into()),
            input_schema: make_schema(
                serde_json::json!({
                    "contact_id": { "type": "string", "description": "UUID of the contact" },
                    "owner_id": { "type": "string", "description": "UUID of the new owner" }
                }),
                vec!["contact_id", "owner_id"],
            ),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
        Tool {
            name: "create_task".into(),
            title: None,
            description: Some("Create a task linked to a contact or deal".into()),
            input_schema: make_schema(
                serde_json::json!({
                    "contact_id": { "type": "string", "description": "UUID of the contact" },
                    "deal_id": { "type": "string", "description": "UUID of the deal" },
                    "title": { "type": "string", "description": "Task title" },
                    "due_date": { "type": "string", "description": "Due date (YYYY-MM-DD)" },
                    "owner_id": { "type": "string", "description": "UUID of task owner" }
                }),
                vec!["title"],
            ),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
        Tool {
            name: "import_contacts".into(),
            title: None,
            description: Some(
                "Bulk import contacts from a JSON array. Each object can have: email, first_name, last_name, company, title, phone, linkedin_url, source, tags"
                    .into(),
            ),
            input_schema: make_schema(
                serde_json::json!({
                    "contacts": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "email": { "type": "string" },
                                "first_name": { "type": "string" },
                                "last_name": { "type": "string" },
                                "company": { "type": "string" },
                                "title": { "type": "string" },
                                "phone": { "type": "string" },
                                "linkedin_url": { "type": "string" },
                                "source": { "type": "string" },
                                "tags": { "type": "array", "items": { "type": "string" } }
                            }
                        },
                        "description": "Array of contact objects to import"
                    }
                }),
                vec!["contacts"],
            ),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
        Tool {
            name: "export_contacts".into(),
            title: None,
            description: Some(
                "Export contacts as JSON array with optional filters by deal stage, owner, or tags"
                    .into(),
            ),
            input_schema: make_schema(
                serde_json::json!({
                    "stage": { "type": "string", "description": "Filter contacts that have deals in this stage" },
                    "owner_id": { "type": "string", "description": "Filter by contact owner UUID" },
                    "tag": { "type": "string", "description": "Filter by tag" }
                }),
                vec![],
            ),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
        Tool {
            name: "add_interaction".into(),
            title: None,
            description: Some(
                "Log an interaction (call, email, meeting, note) with a contact".into(),
            ),
            input_schema: make_schema(
                serde_json::json!({
                    "contact_id": { "type": "string", "description": "UUID of the contact" },
                    "interaction_type": { "type": "string", "enum": ["call", "email", "meeting", "note", "other"], "description": "Interaction type" },
                    "subject": { "type": "string", "description": "Subject/title" },
                    "notes": { "type": "string", "description": "Details" }
                }),
                vec!["contact_id", "interaction_type"],
            ),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
        Tool {
            name: "tag_contact".into(),
            title: None,
            description: Some("Add or remove tags from a contact (stored in contact_tags table)".into()),
            input_schema: make_schema(
                serde_json::json!({
                    "contact_id": { "type": "string", "description": "UUID of the contact" },
                    "add": { "type": "array", "items": { "type": "string" }, "description": "Tags to add" },
                    "remove": { "type": "array", "items": { "type": "string" }, "description": "Tags to remove" }
                }),
                vec!["contact_id"],
            ),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
    ]
}

// ============================================================================
// MCP Server
// ============================================================================

const VALID_STAGES: &[&str] = &[
    "lead",
    "qualified",
    "proposal",
    "negotiation",
    "closed_won",
    "closed_lost",
];

const VALID_ACTIVITY_TYPES: &[&str] = &["call", "email", "meeting", "note"];

#[derive(Clone)]
pub struct CrmMcpServer {
    db: Database,
}

impl CrmMcpServer {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    fn parse_uuid(s: &str) -> Result<uuid::Uuid, CallToolResult> {
        uuid::Uuid::parse_str(s)
            .map_err(|_| error_result(&format!("Invalid UUID: {s}")))
    }

    // ---- Tool handlers ----

    async fn handle_create_contact(&self, args: &serde_json::Value) -> CallToolResult {
        let email = get_str(args, "email");
        let first_name = get_str(args, "first_name");
        let last_name = get_str(args, "last_name");
        let company = get_str(args, "company");
        let title = get_str(args, "title");
        let phone = get_str(args, "phone");
        let linkedin_url = get_str(args, "linkedin_url");
        let source = get_str(args, "source");
        let tags = get_str_array(args, "tags");

        match sqlx::query_as::<_, Contact>(
            r#"INSERT INTO crm.contacts (email, first_name, last_name, company, title, phone, linkedin_url, source, tags)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
               RETURNING *"#,
        )
        .bind(&email)
        .bind(&first_name)
        .bind(&last_name)
        .bind(&company)
        .bind(&title)
        .bind(&phone)
        .bind(&linkedin_url)
        .bind(&source)
        .bind(&tags)
        .fetch_one(self.db.pool())
        .await
        {
            Ok(contact) => {
                info!(email = ?email, "Created contact");
                json_result(&contact)
            }
            Err(e) => error_result(&format!("Failed to create contact: {e}")),
        }
    }

    async fn handle_search_contacts(&self, args: &serde_json::Value) -> CallToolResult {
        let query = match get_str(args, "query") {
            Some(q) => q,
            None => return error_result("Missing required parameter: query"),
        };
        let tag = get_str(args, "tag");
        let limit = get_i64(args, "limit").unwrap_or(50);
        let offset = get_i64(args, "offset").unwrap_or(0);
        let pattern = format!("%{query}%");

        let contacts: Vec<Contact> = if let Some(tag) = &tag {
            match sqlx::query_as::<_, Contact>(
                r#"SELECT * FROM crm.contacts
                   WHERE (first_name ILIKE $1 OR last_name ILIKE $1 OR email ILIKE $1 OR company ILIKE $1)
                     AND $2 = ANY(tags)
                   ORDER BY updated_at DESC
                   LIMIT $3 OFFSET $4"#,
            )
            .bind(&pattern)
            .bind(tag)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.db.pool())
            .await
            {
                Ok(c) => c,
                Err(e) => return error_result(&format!("Search failed: {e}")),
            }
        } else {
            match sqlx::query_as::<_, Contact>(
                r#"SELECT * FROM crm.contacts
                   WHERE first_name ILIKE $1 OR last_name ILIKE $1 OR email ILIKE $1 OR company ILIKE $1
                   ORDER BY updated_at DESC
                   LIMIT $2 OFFSET $3"#,
            )
            .bind(&pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.db.pool())
            .await
            {
                Ok(c) => c,
                Err(e) => return error_result(&format!("Search failed: {e}")),
            }
        };

        json_result(&contacts)
    }

    async fn handle_upsert_deal(&self, args: &serde_json::Value) -> CallToolResult {
        let title = match get_str(args, "title") {
            Some(t) => t,
            None => return error_result("Missing required parameter: title"),
        };
        let contact_id = match get_str(args, "contact_id") {
            Some(s) => match Self::parse_uuid(&s) {
                Ok(u) => Some(u),
                Err(e) => return e,
            },
            None => None,
        };
        let stage = get_str(args, "stage").unwrap_or_else(|| "lead".into());
        if !VALID_STAGES.contains(&stage.as_str()) {
            return error_result(&format!(
                "Invalid stage '{stage}'. Must be one of: {}",
                VALID_STAGES.join(", ")
            ));
        }
        let value = get_f64(args, "value");
        let owner_id = match get_str(args, "owner_id") {
            Some(s) => match Self::parse_uuid(&s) {
                Ok(u) => Some(u),
                Err(e) => return e,
            },
            None => None,
        };
        let expected_close = get_str(args, "expected_close")
            .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok());
        let notes = get_str(args, "notes");

        // Need a unique constraint for upsert. We'll use title + contact_id.
        // First ensure the constraint exists (idempotent).
        let _ = sqlx::raw_sql(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_deals_title_contact ON crm.deals(title, contact_id) WHERE contact_id IS NOT NULL",
        )
        .execute(self.db.pool())
        .await;

        let result = if contact_id.is_some() {
            sqlx::query_as::<_, Deal>(
                r#"INSERT INTO crm.deals (title, contact_id, stage, value, owner_id, expected_close, notes)
                   VALUES ($1, $2, $3, $4, $5, $6, $7)
                   ON CONFLICT (title, contact_id) WHERE contact_id IS NOT NULL
                   DO UPDATE SET stage = EXCLUDED.stage, value = EXCLUDED.value, owner_id = EXCLUDED.owner_id,
                                 expected_close = EXCLUDED.expected_close, notes = EXCLUDED.notes, updated_at = now()
                   RETURNING *"#,
            )
            .bind(&title)
            .bind(&contact_id)
            .bind(&stage)
            .bind(value)
            .bind(&owner_id)
            .bind(&expected_close)
            .bind(&notes)
            .fetch_one(self.db.pool())
            .await
        } else {
            sqlx::query_as::<_, Deal>(
                r#"INSERT INTO crm.deals (title, contact_id, stage, value, owner_id, expected_close, notes)
                   VALUES ($1, $2, $3, $4, $5, $6, $7)
                   RETURNING *"#,
            )
            .bind(&title)
            .bind(&contact_id)
            .bind(&stage)
            .bind(value)
            .bind(&owner_id)
            .bind(&expected_close)
            .bind(&notes)
            .fetch_one(self.db.pool())
            .await
        };

        match result {
            Ok(deal) => {
                info!(title = title, stage = stage, "Upserted deal");
                json_result(&deal)
            }
            Err(e) => error_result(&format!("Failed to upsert deal: {e}")),
        }
    }

    async fn handle_move_deal(&self, args: &serde_json::Value) -> CallToolResult {
        let deal_id_str = match get_str(args, "deal_id") {
            Some(s) => s,
            None => return error_result("Missing required parameter: deal_id"),
        };
        let deal_id = match Self::parse_uuid(&deal_id_str) {
            Ok(u) => u,
            Err(e) => return e,
        };
        let new_stage = match get_str(args, "new_stage") {
            Some(s) => s,
            None => return error_result("Missing required parameter: new_stage"),
        };
        if !VALID_STAGES.contains(&new_stage.as_str()) {
            return error_result(&format!(
                "Invalid stage '{new_stage}'. Must be one of: {}",
                VALID_STAGES.join(", ")
            ));
        }
        let notes = get_str(args, "notes");

        // Get old stage for activity log
        let old: Option<Deal> = match sqlx::query_as(
            "SELECT * FROM crm.deals WHERE id = $1",
        )
        .bind(deal_id)
        .fetch_optional(self.db.pool())
        .await
        {
            Ok(d) => d,
            Err(e) => return error_result(&format!("Database error: {e}")),
        };

        let old = match old {
            Some(d) => d,
            None => return error_result(&format!("Deal '{deal_id}' not found")),
        };

        // Update stage
        let deal: Deal = match sqlx::query_as(
            "UPDATE crm.deals SET stage = $1, updated_at = now() WHERE id = $2 RETURNING *",
        )
        .bind(&new_stage)
        .bind(deal_id)
        .fetch_one(self.db.pool())
        .await
        {
            Ok(d) => d,
            Err(e) => return error_result(&format!("Failed to move deal: {e}")),
        };

        // Auto-log activity
        let subject = format!("Stage changed: {} → {}", old.stage, new_stage);
        let body = notes.unwrap_or_default();
        let _ = sqlx::query(
            "INSERT INTO crm.activities (contact_id, deal_id, activity_type, subject, body) VALUES ($1, $2, 'note', $3, $4)",
        )
        .bind(old.contact_id)
        .bind(deal_id)
        .bind(&subject)
        .bind(&body)
        .execute(self.db.pool())
        .await;

        info!(deal_id = %deal_id, old_stage = old.stage, new_stage = new_stage, "Moved deal");
        json_result(&deal)
    }

    async fn handle_log_activity(&self, args: &serde_json::Value) -> CallToolResult {
        let contact_id = match get_str(args, "contact_id") {
            Some(s) => match Self::parse_uuid(&s) {
                Ok(u) => Some(u),
                Err(e) => return e,
            },
            None => None,
        };
        let deal_id = match get_str(args, "deal_id") {
            Some(s) => match Self::parse_uuid(&s) {
                Ok(u) => Some(u),
                Err(e) => return e,
            },
            None => None,
        };
        let activity_type = match get_str(args, "activity_type") {
            Some(t) => t,
            None => return error_result("Missing required parameter: activity_type"),
        };
        if !VALID_ACTIVITY_TYPES.contains(&activity_type.as_str()) {
            return error_result(&format!(
                "Invalid activity_type '{activity_type}'. Must be one of: {}",
                VALID_ACTIVITY_TYPES.join(", ")
            ));
        }
        let subject = get_str(args, "subject");
        let body = get_str(args, "body");

        match sqlx::query_as::<_, Activity>(
            r#"INSERT INTO crm.activities (contact_id, deal_id, activity_type, subject, body)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING *"#,
        )
        .bind(&contact_id)
        .bind(&deal_id)
        .bind(&activity_type)
        .bind(&subject)
        .bind(&body)
        .fetch_one(self.db.pool())
        .await
        {
            Ok(activity) => {
                info!(activity_type = activity_type, "Logged activity");
                json_result(&activity)
            }
            Err(e) => error_result(&format!("Failed to log activity: {e}")),
        }
    }

    async fn handle_get_pipeline(&self, args: &serde_json::Value) -> CallToolResult {
        let owner_id = match get_str(args, "owner_id") {
            Some(s) => match Self::parse_uuid(&s) {
                Ok(u) => Some(u),
                Err(e) => return e,
            },
            None => None,
        };

        let rows: Vec<PipelineRow> = if let Some(oid) = owner_id {
            match sqlx::query_as::<_, PipelineRow>(
                r#"SELECT stage, COUNT(*) as deal_count, SUM(value) as total_value
                   FROM crm.deals WHERE owner_id = $1
                   GROUP BY stage ORDER BY stage"#,
            )
            .bind(oid)
            .fetch_all(self.db.pool())
            .await
            {
                Ok(r) => r,
                Err(e) => return error_result(&format!("Pipeline query failed: {e}")),
            }
        } else {
            match sqlx::query_as::<_, PipelineRow>(
                r#"SELECT stage, COUNT(*) as deal_count, SUM(value) as total_value
                   FROM crm.deals
                   GROUP BY stage ORDER BY stage"#,
            )
            .fetch_all(self.db.pool())
            .await
            {
                Ok(r) => r,
                Err(e) => return error_result(&format!("Pipeline query failed: {e}")),
            }
        };

        json_result(&rows)
    }

    async fn handle_assign_contact(&self, args: &serde_json::Value) -> CallToolResult {
        let contact_id_str = match get_str(args, "contact_id") {
            Some(s) => s,
            None => return error_result("Missing required parameter: contact_id"),
        };
        let contact_id = match Self::parse_uuid(&contact_id_str) {
            Ok(u) => u,
            Err(e) => return e,
        };
        let owner_id_str = match get_str(args, "owner_id") {
            Some(s) => s,
            None => return error_result("Missing required parameter: owner_id"),
        };
        let owner_id = match Self::parse_uuid(&owner_id_str) {
            Ok(u) => u,
            Err(e) => return e,
        };

        match sqlx::query_as::<_, Contact>(
            "UPDATE crm.contacts SET owner_id = $1, updated_at = now() WHERE id = $2 RETURNING *",
        )
        .bind(owner_id)
        .bind(contact_id)
        .fetch_optional(self.db.pool())
        .await
        {
            Ok(Some(contact)) => {
                info!(contact_id = %contact_id, owner_id = %owner_id, "Assigned contact");
                json_result(&contact)
            }
            Ok(None) => error_result(&format!("Contact '{contact_id}' not found")),
            Err(e) => error_result(&format!("Failed to assign contact: {e}")),
        }
    }

    async fn handle_create_task(&self, args: &serde_json::Value) -> CallToolResult {
        let title = match get_str(args, "title") {
            Some(t) => t,
            None => return error_result("Missing required parameter: title"),
        };
        let contact_id = match get_str(args, "contact_id") {
            Some(s) => match Self::parse_uuid(&s) {
                Ok(u) => Some(u),
                Err(e) => return e,
            },
            None => None,
        };
        let deal_id = match get_str(args, "deal_id") {
            Some(s) => match Self::parse_uuid(&s) {
                Ok(u) => Some(u),
                Err(e) => return e,
            },
            None => None,
        };
        let due_date = get_str(args, "due_date")
            .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok());
        let owner_id = match get_str(args, "owner_id") {
            Some(s) => match Self::parse_uuid(&s) {
                Ok(u) => Some(u),
                Err(e) => return e,
            },
            None => None,
        };

        match sqlx::query_as::<_, Task>(
            r#"INSERT INTO crm.tasks (contact_id, deal_id, title, due_date, owner_id)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING *"#,
        )
        .bind(&contact_id)
        .bind(&deal_id)
        .bind(&title)
        .bind(&due_date)
        .bind(&owner_id)
        .fetch_one(self.db.pool())
        .await
        {
            Ok(task) => {
                info!(title = title, "Created task");
                json_result(&task)
            }
            Err(e) => error_result(&format!("Failed to create task: {e}")),
        }
    }

    async fn handle_import_contacts(&self, args: &serde_json::Value) -> CallToolResult {
        let contacts_arr = match args.get("contacts").and_then(|v| v.as_array()) {
            Some(a) => a,
            None => {
                return error_result(
                    "Missing required parameter: contacts (must be a JSON array)",
                )
            }
        };

        let mut imported = 0i64;
        let mut errors: Vec<String> = Vec::new();

        for (i, c) in contacts_arr.iter().enumerate() {
            let email = c.get("email").and_then(|v| v.as_str());
            let first_name = c.get("first_name").and_then(|v| v.as_str());
            let last_name = c.get("last_name").and_then(|v| v.as_str());
            let company = c.get("company").and_then(|v| v.as_str());
            let title = c.get("title").and_then(|v| v.as_str());
            let phone = c.get("phone").and_then(|v| v.as_str());
            let linkedin_url = c.get("linkedin_url").and_then(|v| v.as_str());
            let source = c.get("source").and_then(|v| v.as_str());
            let tags: Vec<String> = c
                .get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            match sqlx::query(
                r#"INSERT INTO crm.contacts (email, first_name, last_name, company, title, phone, linkedin_url, source, tags)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                   ON CONFLICT (email) DO NOTHING"#,
            )
            .bind(email)
            .bind(first_name)
            .bind(last_name)
            .bind(company)
            .bind(title)
            .bind(phone)
            .bind(linkedin_url)
            .bind(source)
            .bind(&tags)
            .execute(self.db.pool())
            .await
            {
                Ok(r) => {
                    if r.rows_affected() > 0 {
                        imported += 1;
                    }
                }
                Err(e) => {
                    errors.push(format!("Row {i}: {e}"));
                }
            }
        }

        info!(imported = imported, total = contacts_arr.len(), "Imported contacts");
        json_result(&serde_json::json!({
            "imported": imported,
            "total": contacts_arr.len(),
            "errors": errors
        }))
    }

    async fn handle_export_contacts(&self, args: &serde_json::Value) -> CallToolResult {
        let stage = get_str(args, "stage");
        let owner_id = match get_str(args, "owner_id") {
            Some(s) => match Self::parse_uuid(&s) {
                Ok(u) => Some(u),
                Err(e) => return e,
            },
            None => None,
        };
        let tag = get_str(args, "tag");

        // Build dynamic query
        let mut sql = String::from("SELECT DISTINCT c.* FROM crm.contacts c");
        let mut conditions: Vec<String> = Vec::new();
        let mut has_deal_join = false;

        if stage.is_some() {
            sql.push_str(" JOIN crm.deals d ON d.contact_id = c.id");
            has_deal_join = true;
            conditions.push("placeholder_stage".to_string());
        }

        if owner_id.is_some() {
            conditions.push("placeholder_owner".to_string());
        }

        if tag.is_some() {
            conditions.push("placeholder_tag".to_string());
        }

        // Rebuild with positional params
        let mut final_sql = String::from("SELECT DISTINCT c.* FROM crm.contacts c");
        if has_deal_join {
            final_sql.push_str(" JOIN crm.deals d ON d.contact_id = c.id");
        }

        let mut param_idx = 1;
        let mut where_parts: Vec<String> = Vec::new();

        if stage.is_some() {
            where_parts.push(format!("d.stage = ${param_idx}"));
            param_idx += 1;
        }
        if owner_id.is_some() {
            where_parts.push(format!("c.owner_id = ${param_idx}"));
            param_idx += 1;
        }
        if tag.is_some() {
            where_parts.push(format!("${param_idx} = ANY(c.tags)"));
        }

        if !where_parts.is_empty() {
            final_sql.push_str(" WHERE ");
            final_sql.push_str(&where_parts.join(" AND "));
        }
        final_sql.push_str(" ORDER BY c.updated_at DESC");

        // Use a dynamic query approach
        let mut query = sqlx::query_as::<_, Contact>(&final_sql);
        if let Some(ref s) = stage {
            query = query.bind(s);
        }
        if let Some(oid) = owner_id {
            query = query.bind(oid);
        }
        if let Some(ref t) = tag {
            query = query.bind(t);
        }

        match query.fetch_all(self.db.pool()).await {
            Ok(contacts) => json_result(&contacts),
            Err(e) => error_result(&format!("Export failed: {e}")),
        }
    }

    async fn handle_add_interaction(&self, args: &serde_json::Value) -> CallToolResult {
        let contact_id_str = match get_str(args, "contact_id") {
            Some(s) => s,
            None => return error_result("Missing required parameter: contact_id"),
        };
        let contact_id = match Self::parse_uuid(&contact_id_str) {
            Ok(u) => u,
            Err(e) => return e,
        };
        let interaction_type = match get_str(args, "interaction_type") {
            Some(t) => t,
            None => return error_result("Missing required parameter: interaction_type"),
        };

        // Verify contact exists
        let exists: Option<(uuid::Uuid,)> =
            match sqlx::query_as("SELECT id FROM crm.contacts WHERE id = $1")
                .bind(contact_id)
                .fetch_optional(self.db.pool())
                .await
            {
                Ok(c) => c,
                Err(e) => return error_result(&format!("Database error: {e}")),
            };
        if exists.is_none() {
            return error_result(&format!("Contact '{contact_id}' not found"));
        }

        let subject = get_str(args, "subject");
        let notes = get_str(args, "notes");

        match sqlx::query_as::<_, ContactInteraction>(
            "INSERT INTO crm.contact_interactions (contact_id, interaction_type, subject, notes) \
             VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(contact_id)
        .bind(&interaction_type)
        .bind(&subject)
        .bind(&notes)
        .fetch_one(self.db.pool())
        .await
        {
            Ok(interaction) => {
                info!(contact_id = %contact_id, interaction_type = interaction_type, "Added interaction");
                json_result(&interaction)
            }
            Err(e) => error_result(&format!("Failed to add interaction: {e}")),
        }
    }

    async fn handle_tag_contact(&self, args: &serde_json::Value) -> CallToolResult {
        let contact_id_str = match get_str(args, "contact_id") {
            Some(s) => s,
            None => return error_result("Missing required parameter: contact_id"),
        };
        let contact_id = match Self::parse_uuid(&contact_id_str) {
            Ok(u) => u,
            Err(e) => return e,
        };

        // Verify contact exists
        let exists: Option<(uuid::Uuid,)> =
            match sqlx::query_as("SELECT id FROM crm.contacts WHERE id = $1")
                .bind(contact_id)
                .fetch_optional(self.db.pool())
                .await
            {
                Ok(c) => c,
                Err(e) => return error_result(&format!("Database error: {e}")),
            };
        if exists.is_none() {
            return error_result(&format!("Contact '{contact_id}' not found"));
        }

        let add_tags = get_str_array(args, "add");
        let remove_tags = get_str_array(args, "remove");

        for tag in &add_tags {
            let _ = sqlx::query(
                "INSERT INTO crm.contact_tags (contact_id, tag) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(contact_id)
            .bind(tag)
            .execute(self.db.pool())
            .await;
        }

        for tag in &remove_tags {
            let _ = sqlx::query(
                "DELETE FROM crm.contact_tags WHERE contact_id = $1 AND tag = $2",
            )
            .bind(contact_id)
            .bind(tag)
            .execute(self.db.pool())
            .await;
        }

        // Return current tags
        let tags: Vec<ContactTag> = sqlx::query_as(
            "SELECT * FROM crm.contact_tags WHERE contact_id = $1 ORDER BY tag",
        )
        .bind(contact_id)
        .fetch_all(self.db.pool())
        .await
        .unwrap_or_default();

        json_result(&serde_json::json!({
            "contact_id": contact_id,
            "tags": tags.iter().map(|t| &t.tag).collect::<Vec<_>>(),
            "added": add_tags,
            "removed": remove_tags
        }))
    }
}

// ============================================================================
// ServerHandler trait implementation
// ============================================================================

impl ServerHandler for CrmMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "DataXLR8 CRM MCP — manage contacts, deals, pipeline, activities, and tasks"
                    .into(),
            ),
        }
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, rmcp::ErrorData>> + Send + '_
    {
        async {
            Ok(ListToolsResult {
                tools: build_tools(),
                next_cursor: None,
                meta: None,
            })
        }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, rmcp::ErrorData>> + Send + '_
    {
        async move {
            let args =
                serde_json::to_value(&request.arguments).unwrap_or(serde_json::Value::Null);
            let name_str: &str = request.name.as_ref();

            let result = match name_str {
                "create_contact" => self.handle_create_contact(&args).await,
                "search_contacts" => self.handle_search_contacts(&args).await,
                "upsert_deal" => self.handle_upsert_deal(&args).await,
                "move_deal" => self.handle_move_deal(&args).await,
                "log_activity" => self.handle_log_activity(&args).await,
                "get_pipeline" => self.handle_get_pipeline(&args).await,
                "assign_contact" => self.handle_assign_contact(&args).await,
                "create_task" => self.handle_create_task(&args).await,
                "import_contacts" => self.handle_import_contacts(&args).await,
                "export_contacts" => self.handle_export_contacts(&args).await,
                "add_interaction" => self.handle_add_interaction(&args).await,
                "tag_contact" => self.handle_tag_contact(&args).await,
                _ => error_result(&format!("Unknown tool: {}", request.name)),
            };

            Ok(result)
        }
    }
}
