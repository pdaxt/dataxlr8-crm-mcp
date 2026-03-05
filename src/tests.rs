use dataxlr8_mcp_core::mcp::{get_i64, get_str, get_str_array};
use serde_json::json;

// ============================================================================
// Validation logic (mirrors production validators for unit testing)
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

fn is_valid_stage(stage: &str) -> bool {
    VALID_STAGES.contains(&stage)
}

fn is_valid_activity_type(activity_type: &str) -> bool {
    VALID_ACTIVITY_TYPES.contains(&activity_type)
}

fn parse_uuid(s: &str) -> Result<uuid::Uuid, String> {
    uuid::Uuid::parse_str(s).map_err(|e| format!("Invalid UUID: {s} — {e}"))
}

// ============================================================================
// UUID parsing — valid
// ============================================================================

#[test]
fn uuid_valid_v4() {
    let id = uuid::Uuid::new_v4();
    assert!(parse_uuid(&id.to_string()).is_ok());
}

#[test]
fn uuid_valid_hyphenated() {
    assert!(parse_uuid("550e8400-e29b-41d4-a716-446655440000").is_ok());
}

#[test]
fn uuid_valid_uppercase() {
    assert!(parse_uuid("550E8400-E29B-41D4-A716-446655440000").is_ok());
}

#[test]
fn uuid_valid_nil() {
    assert!(parse_uuid("00000000-0000-0000-0000-000000000000").is_ok());
}

// ============================================================================
// UUID parsing — invalid
// ============================================================================

#[test]
fn uuid_empty_string() {
    assert!(parse_uuid("").is_err());
}

#[test]
fn uuid_not_a_uuid() {
    assert!(parse_uuid("not-a-uuid").is_err());
}

#[test]
fn uuid_too_short() {
    assert!(parse_uuid("550e8400-e29b-41d4").is_err());
}

#[test]
fn uuid_too_long() {
    assert!(parse_uuid("550e8400-e29b-41d4-a716-446655440000-extra").is_err());
}

#[test]
fn uuid_invalid_chars() {
    assert!(parse_uuid("550e8400-e29b-41d4-a716-44665544ZZZZ").is_err());
}

#[test]
fn uuid_sql_injection() {
    assert!(parse_uuid("'; DROP TABLE crm.contacts;--").is_err());
    assert!(parse_uuid("550e8400' OR '1'='1").is_err());
    assert!(parse_uuid("1; SELECT * FROM users").is_err());
}

#[test]
fn uuid_with_whitespace() {
    assert!(parse_uuid(" 550e8400-e29b-41d4-a716-446655440000").is_err());
    assert!(parse_uuid("550e8400-e29b-41d4-a716-446655440000 ").is_err());
    assert!(parse_uuid("550e8400 -e29b-41d4-a716-446655440000").is_err());
}

#[test]
fn uuid_with_null_byte() {
    assert!(parse_uuid("550e8400-e29b-41d4-a716-\0446655440000").is_err());
}

#[test]
fn uuid_very_long_string() {
    let long = "a".repeat(1000);
    assert!(parse_uuid(&long).is_err());
}

#[test]
fn uuid_numeric_only() {
    assert!(parse_uuid("12345678901234567890123456789012").is_ok()); // 32 hex chars, no hyphens — valid for uuid
}

#[test]
fn uuid_with_braces() {
    // uuid crate supports braced format
    assert!(parse_uuid("{550e8400-e29b-41d4-a716-446655440000}").is_ok());
}

#[test]
fn uuid_unicode() {
    assert!(parse_uuid("日本語テスト").is_err());
}

// ============================================================================
// Stage validation
// ============================================================================

#[test]
fn stage_valid_all() {
    for stage in VALID_STAGES {
        assert!(is_valid_stage(stage), "Stage '{stage}' should be valid");
    }
}

#[test]
fn stage_empty_string() {
    assert!(!is_valid_stage(""));
}

#[test]
fn stage_invalid() {
    assert!(!is_valid_stage("won"));
    assert!(!is_valid_stage("lost"));
    assert!(!is_valid_stage("pending"));
    assert!(!is_valid_stage("active"));
}

#[test]
fn stage_case_sensitive() {
    assert!(!is_valid_stage("Lead"));
    assert!(!is_valid_stage("LEAD"));
    assert!(!is_valid_stage("Closed_Won"));
}

#[test]
fn stage_with_whitespace() {
    assert!(!is_valid_stage(" lead"));
    assert!(!is_valid_stage("lead "));
    assert!(!is_valid_stage(" lead "));
}

#[test]
fn stage_sql_injection() {
    assert!(!is_valid_stage("lead'; DROP TABLE crm.deals;--"));
    assert!(!is_valid_stage("' OR '1'='1"));
}

#[test]
fn stage_very_long_string() {
    let long = "x".repeat(1000);
    assert!(!is_valid_stage(&long));
}

#[test]
fn stage_null_bytes() {
    assert!(!is_valid_stage("lead\0"));
    assert!(!is_valid_stage("\0lead"));
}

// ============================================================================
// Activity type validation
// ============================================================================

#[test]
fn activity_type_valid_all() {
    for at in VALID_ACTIVITY_TYPES {
        assert!(is_valid_activity_type(at), "Activity type '{at}' should be valid");
    }
}

#[test]
fn activity_type_empty() {
    assert!(!is_valid_activity_type(""));
}

#[test]
fn activity_type_invalid() {
    assert!(!is_valid_activity_type("sms"));
    assert!(!is_valid_activity_type("chat"));
    assert!(!is_valid_activity_type("task"));
}

#[test]
fn activity_type_case_sensitive() {
    assert!(!is_valid_activity_type("Call"));
    assert!(!is_valid_activity_type("EMAIL"));
    assert!(!is_valid_activity_type("Meeting"));
}

#[test]
fn activity_type_sql_injection() {
    assert!(!is_valid_activity_type("call'; DROP TABLE crm.activities;--"));
}

// ============================================================================
// Date parsing (mirrors expected_close / due_date parsing)
// ============================================================================

#[test]
fn date_valid() {
    let date = chrono::NaiveDate::parse_from_str("2024-01-15", "%Y-%m-%d");
    assert!(date.is_ok());
}

#[test]
fn date_empty_string() {
    let date = chrono::NaiveDate::parse_from_str("", "%Y-%m-%d");
    assert!(date.is_err());
}

#[test]
fn date_invalid_format() {
    let date = chrono::NaiveDate::parse_from_str("01/15/2024", "%Y-%m-%d");
    assert!(date.is_err());
}

#[test]
fn date_invalid_month() {
    let date = chrono::NaiveDate::parse_from_str("2024-13-01", "%Y-%m-%d");
    assert!(date.is_err());
}

#[test]
fn date_invalid_day() {
    let date = chrono::NaiveDate::parse_from_str("2024-02-30", "%Y-%m-%d");
    assert!(date.is_err());
}

#[test]
fn date_leap_year_valid() {
    let date = chrono::NaiveDate::parse_from_str("2024-02-29", "%Y-%m-%d");
    assert!(date.is_ok());
}

#[test]
fn date_leap_year_invalid() {
    let date = chrono::NaiveDate::parse_from_str("2023-02-29", "%Y-%m-%d");
    assert!(date.is_err());
}

#[test]
fn date_sql_injection() {
    let date = chrono::NaiveDate::parse_from_str("2024-01-01'; DROP TABLE crm.deals;--", "%Y-%m-%d");
    assert!(date.is_err());
}

#[test]
fn date_very_far_future() {
    let date = chrono::NaiveDate::parse_from_str("9999-12-31", "%Y-%m-%d");
    assert!(date.is_ok());
}

#[test]
fn date_very_far_past() {
    let date = chrono::NaiveDate::parse_from_str("0001-01-01", "%Y-%m-%d");
    assert!(date.is_ok());
}

// ============================================================================
// Search contacts — pattern construction
// ============================================================================

#[test]
fn search_pattern_basic() {
    let query = "john";
    let pattern = format!("%{query}%");
    assert_eq!(pattern, "%john%");
}

#[test]
fn search_pattern_empty_query() {
    let query = "";
    let pattern = format!("%{query}%");
    assert_eq!(pattern, "%%"); // Matches everything — valid behavior
}

#[test]
fn search_pattern_very_long_query() {
    let query = "a".repeat(1000);
    let pattern = format!("%{query}%");
    assert_eq!(pattern.len(), 1002);
}

#[test]
fn search_pattern_sql_injection() {
    // NOTE: The CRM search_contacts does NOT escape LIKE metacharacters
    // (unlike enrichment's search_people which does). This means % and _ in
    // the query will act as ILIKE wildcards. However, actual SQL injection
    // is still prevented by parameterized queries ($1).
    let query = "'; DROP TABLE crm.contacts;--";
    let pattern = format!("%{query}%");
    assert!(pattern.contains("DROP TABLE")); // Value passed to parameterized query
}

#[test]
fn search_pattern_ilike_wildcards() {
    // CRM does not escape these — they'll be active ILIKE wildcards
    let query = "jo%n";
    let pattern = format!("%{query}%");
    assert!(pattern.contains('%'));
}

// ============================================================================
// Search contacts — limit/offset
// ============================================================================

#[test]
fn search_limit_default_value() {
    let args = json!({"query": "test"});
    let limit = get_i64(&args, "limit").unwrap_or(50);
    assert_eq!(limit, 50);
}

#[test]
fn search_limit_zero() {
    let args = json!({"query": "test", "limit": 0});
    let limit = get_i64(&args, "limit").unwrap_or(50);
    assert_eq!(limit, 0);
}

#[test]
fn search_limit_negative() {
    let args = json!({"query": "test", "limit": -1});
    let limit = get_i64(&args, "limit").unwrap_or(50);
    assert_eq!(limit, -1); // NOTE: Not clamped in CRM — passed directly to SQL LIMIT
}

#[test]
fn search_limit_i64_max() {
    let args = json!({"query": "test", "limit": i64::MAX});
    let limit = get_i64(&args, "limit").unwrap_or(50);
    assert_eq!(limit, i64::MAX); // NOTE: Not clamped in CRM
}

#[test]
fn search_offset_default() {
    let args = json!({"query": "test"});
    let offset = get_i64(&args, "offset").unwrap_or(0);
    assert_eq!(offset, 0);
}

#[test]
fn search_offset_negative() {
    let args = json!({"query": "test", "offset": -1});
    let offset = get_i64(&args, "offset").unwrap_or(0);
    assert_eq!(offset, -1); // NOTE: Not clamped in CRM
}

#[test]
fn search_offset_i64_max() {
    let args = json!({"query": "test", "offset": i64::MAX});
    let offset = get_i64(&args, "offset").unwrap_or(0);
    assert_eq!(offset, i64::MAX);
}

// ============================================================================
// Import contacts — edge cases
// ============================================================================

#[test]
fn import_contacts_empty_array() {
    let args = json!({"contacts": []});
    let contacts = args.get("contacts").and_then(|v| v.as_array());
    assert!(contacts.unwrap().is_empty());
}

#[test]
fn import_contacts_missing_field() {
    let args = json!({});
    let contacts = args.get("contacts").and_then(|v| v.as_array());
    assert!(contacts.is_none());
}

#[test]
fn import_contacts_not_array() {
    let args = json!({"contacts": "not-an-array"});
    let contacts = args.get("contacts").and_then(|v| v.as_array());
    assert!(contacts.is_none());
}

#[test]
fn import_contacts_null_value() {
    let args = json!({"contacts": null});
    let contacts = args.get("contacts").and_then(|v| v.as_array());
    assert!(contacts.is_none());
}

#[test]
fn import_contacts_empty_objects() {
    let args = json!({"contacts": [{}, {}, {}]});
    let contacts = args.get("contacts").and_then(|v| v.as_array()).unwrap();
    assert_eq!(contacts.len(), 3);
    // Each contact has no fields — all values will be None
    for c in contacts {
        assert!(c.get("email").is_none());
        assert!(c.get("first_name").is_none());
    }
}

#[test]
fn import_contacts_sql_injection_in_fields() {
    let args = json!({"contacts": [{
        "email": "'; DROP TABLE crm.contacts;--@evil.com",
        "first_name": "Robert'); DROP TABLE crm.contacts;--",
        "company": "' OR '1'='1"
    }]});
    let contacts = args.get("contacts").and_then(|v| v.as_array()).unwrap();
    let c = &contacts[0];
    let email = c.get("email").and_then(|v| v.as_str()).unwrap();
    assert!(email.contains("DROP TABLE")); // Passed to parameterized query
}

#[test]
fn import_contacts_very_long_fields() {
    let long_name = "x".repeat(5000);
    let args = json!({"contacts": [{
        "first_name": long_name,
        "last_name": "y".repeat(5000),
        "email": format!("{}@example.com", "a".repeat(1000))
    }]});
    let contacts = args.get("contacts").and_then(|v| v.as_array()).unwrap();
    let name = contacts[0].get("first_name").and_then(|v| v.as_str()).unwrap();
    assert_eq!(name.len(), 5000);
}

#[test]
fn import_contacts_unicode_fields() {
    let args = json!({"contacts": [{
        "first_name": "München",
        "last_name": "日本語",
        "company": "Ñoño Industries"
    }]});
    let contacts = args.get("contacts").and_then(|v| v.as_array()).unwrap();
    let name = contacts[0].get("first_name").and_then(|v| v.as_str()).unwrap();
    assert_eq!(name, "München");
}

#[test]
fn import_contacts_special_chars_in_tags() {
    let args = json!({"contacts": [{
        "email": "test@example.com",
        "tags": ["tag with spaces", "tag\"quotes", "tag\\backslash", "tag\nnewline"]
    }]});
    let contacts = args.get("contacts").and_then(|v| v.as_array()).unwrap();
    let tags = contacts[0].get("tags").and_then(|v| v.as_array()).unwrap();
    assert_eq!(tags.len(), 4);
}

#[test]
fn import_contacts_duplicate_emails() {
    let args = json!({"contacts": [
        {"email": "dupe@example.com", "first_name": "First"},
        {"email": "dupe@example.com", "first_name": "Second"}
    ]});
    let contacts = args.get("contacts").and_then(|v| v.as_array()).unwrap();
    assert_eq!(contacts.len(), 2);
    // Both have same email — ON CONFLICT DO NOTHING in production
}

// ============================================================================
// Export contacts — dynamic query building
// ============================================================================

#[test]
fn export_stage_filter_valid() {
    let args = json!({"stage": "lead"});
    let stage = get_str(&args, "stage");
    assert_eq!(stage, Some("lead".to_string()));
    assert!(is_valid_stage("lead"));
}

#[test]
fn export_stage_filter_invalid() {
    let args = json!({"stage": "nonexistent"});
    let stage = get_str(&args, "stage").unwrap();
    assert!(!is_valid_stage(&stage));
}

#[test]
fn export_owner_filter_invalid_uuid() {
    let args = json!({"owner_id": "not-a-uuid"});
    let owner = get_str(&args, "owner_id").unwrap();
    assert!(parse_uuid(&owner).is_err());
}

#[test]
fn export_tag_filter_sql_injection() {
    let args = json!({"tag": "'; DROP TABLE crm.contacts;--"});
    let tag = get_str(&args, "tag").unwrap();
    assert!(tag.contains("DROP TABLE")); // Safe — parameterized query
}

#[test]
fn export_no_filters() {
    let args = json!({});
    assert!(get_str(&args, "stage").is_none());
    assert!(get_str(&args, "owner_id").is_none());
    assert!(get_str(&args, "tag").is_none());
}

// ============================================================================
// Tag operations — edge cases
// ============================================================================

#[test]
fn tag_add_empty_array() {
    let args = json!({"contact_id": "550e8400-e29b-41d4-a716-446655440000", "add": []});
    let add_tags = get_str_array(&args, "add");
    assert!(add_tags.is_empty());
}

#[test]
fn tag_add_and_remove_same() {
    let args = json!({
        "contact_id": "550e8400-e29b-41d4-a716-446655440000",
        "add": ["vip"],
        "remove": ["vip"]
    });
    let add_tags = get_str_array(&args, "add");
    let remove_tags = get_str_array(&args, "remove");
    // Both contain "vip" — behavior depends on execution order
    assert_eq!(add_tags, vec!["vip"]);
    assert_eq!(remove_tags, vec!["vip"]);
}

#[test]
fn tag_special_characters() {
    let args = json!({
        "contact_id": "550e8400-e29b-41d4-a716-446655440000",
        "add": ["tag with spaces", "tag-with-dashes", "tag_underscores", "UPPERCASE", "MiXeD"]
    });
    let add_tags = get_str_array(&args, "add");
    assert_eq!(add_tags.len(), 5);
}

#[test]
fn tag_sql_injection() {
    let args = json!({
        "contact_id": "550e8400-e29b-41d4-a716-446655440000",
        "add": ["'; DROP TABLE crm.contact_tags;--"]
    });
    let add_tags = get_str_array(&args, "add");
    assert_eq!(add_tags[0], "'; DROP TABLE crm.contact_tags;--");
}

#[test]
fn tag_very_long() {
    let long_tag = "x".repeat(1000);
    let args = json!({
        "contact_id": "550e8400-e29b-41d4-a716-446655440000",
        "add": [long_tag]
    });
    let add_tags = get_str_array(&args, "add");
    assert_eq!(add_tags[0].len(), 1000);
}

#[test]
fn tag_unicode() {
    let args = json!({
        "contact_id": "550e8400-e29b-41d4-a716-446655440000",
        "add": ["高优先级", "VIP客户"]
    });
    let add_tags = get_str_array(&args, "add");
    assert_eq!(add_tags.len(), 2);
    assert_eq!(add_tags[0], "高优先级");
}

// ============================================================================
// Core helper edge cases (CRM-specific usage)
// ============================================================================

#[test]
fn get_str_deal_value_as_number() {
    // In upsert_deal, value is extracted with get_f64, not get_str
    let args = json!({"title": "Big Deal", "value": 50000.00});
    assert!(get_str(&args, "value").is_none()); // It's a number, not string
}

#[test]
fn get_str_nested_object() {
    let args = json!({"custom_fields": {"key": "value"}});
    assert!(get_str(&args, "custom_fields").is_none()); // Objects aren't strings
}

#[test]
fn args_null_root() {
    let args = serde_json::Value::Null;
    assert!(get_str(&args, "title").is_none());
    assert!(get_i64(&args, "limit").is_none());
    assert!(get_str_array(&args, "tags").is_empty());
}

#[test]
fn args_array_root() {
    let args = json!(["not", "an", "object"]);
    assert!(get_str(&args, "0").is_none());
}
