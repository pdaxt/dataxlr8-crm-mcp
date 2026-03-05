use anyhow::Result;
use sqlx::PgPool;

/// Create the CRM schema in PostgreSQL if it doesn't exist.
pub async fn setup_schema(pool: &PgPool) -> Result<()> {
    sqlx::raw_sql(
        r#"
        CREATE SCHEMA IF NOT EXISTS crm;

        CREATE TABLE IF NOT EXISTS crm.contacts (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            email TEXT UNIQUE,
            first_name TEXT,
            last_name TEXT,
            company TEXT,
            title TEXT,
            phone TEXT,
            linkedin_url TEXT,
            source TEXT,
            tags TEXT[] DEFAULT ARRAY[]::TEXT[],
            custom_fields JSONB DEFAULT '{}'::jsonb,
            owner_id UUID,
            created_at TIMESTAMPTZ DEFAULT now(),
            updated_at TIMESTAMPTZ DEFAULT now()
        );

        CREATE TABLE IF NOT EXISTS crm.deals (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            title TEXT NOT NULL,
            contact_id UUID REFERENCES crm.contacts(id),
            stage TEXT NOT NULL DEFAULT 'lead',
            value NUMERIC(12,2) DEFAULT 0,
            owner_id UUID,
            expected_close DATE,
            notes TEXT,
            created_at TIMESTAMPTZ DEFAULT now(),
            updated_at TIMESTAMPTZ DEFAULT now()
        );

        CREATE TABLE IF NOT EXISTS crm.activities (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            contact_id UUID REFERENCES crm.contacts(id),
            deal_id UUID REFERENCES crm.deals(id),
            activity_type TEXT NOT NULL,
            subject TEXT,
            body TEXT,
            occurred_at TIMESTAMPTZ DEFAULT now()
        );

        CREATE TABLE IF NOT EXISTS crm.tasks (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            contact_id UUID REFERENCES crm.contacts(id),
            deal_id UUID REFERENCES crm.deals(id),
            title TEXT NOT NULL,
            due_date DATE,
            completed BOOLEAN DEFAULT false,
            owner_id UUID,
            created_at TIMESTAMPTZ DEFAULT now()
        );

        CREATE INDEX IF NOT EXISTS idx_contacts_email ON crm.contacts(email);
        CREATE INDEX IF NOT EXISTS idx_contacts_company ON crm.contacts(company);
        CREATE INDEX IF NOT EXISTS idx_deals_stage ON crm.deals(stage);
        CREATE INDEX IF NOT EXISTS idx_deals_contact ON crm.deals(contact_id);
        CREATE INDEX IF NOT EXISTS idx_activities_contact ON crm.activities(contact_id);
        CREATE INDEX IF NOT EXISTS idx_tasks_owner ON crm.tasks(owner_id);
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
