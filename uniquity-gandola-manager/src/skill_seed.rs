//! Idempotent seeds for Gandola LLM assistant skills.

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

use lariv_rs::plugins::llm_assistant::entities::skill::{self, Entity as SkillEntity};

const INVOICE_SITE_POS_SKILL: &str =
    include_str!("../skills/invoice-site-purchase-orders/index.json");
const CREATE_PO_FROM_PDF_SKILL: &str =
    include_str!("../skills/create-purchase-order-from-pdf/index.json");

#[derive(serde::Deserialize)]
struct SkillExport {
    name: String,
    description: String,
    content: String,
}

async fn ensure_skill(db: &DatabaseConnection, json: &str) -> anyhow::Result<()> {
    let export: SkillExport = serde_json::from_str(json)?;
    if let Some(existing) = SkillEntity::find()
        .filter(skill::Column::Name.eq(&export.name))
        .one(db)
        .await?
    {
        if existing.content != export.content || existing.description != export.description {
            let mut am: skill::ActiveModel = existing.into();
            am.description = Set(export.description);
            am.content = Set(export.content);
            am.updated_at = Set(Some(Utc::now()));
            am.update(db).await?;
        }
        return Ok(());
    }
    skill::ActiveModel {
        name: Set(export.name),
        description: Set(export.description),
        content: Set(export.content),
        created_at: Set(Some(Utc::now())),
        updated_at: Set(Some(Utc::now())),
        ..Default::default()
    }
    .insert(db)
    .await?;
    Ok(())
}

pub async fn ensure_invoice_site_pos_skill(db: &DatabaseConnection) -> anyhow::Result<()> {
    ensure_skill(db, INVOICE_SITE_POS_SKILL).await
}

pub async fn ensure_create_po_from_pdf_skill(db: &DatabaseConnection) -> anyhow::Result<()> {
    ensure_skill(db, CREATE_PO_FROM_PDF_SKILL).await
}

pub async fn ensure_all_skills(db: &DatabaseConnection) -> anyhow::Result<()> {
    ensure_invoice_site_pos_skill(db).await?;
    ensure_create_po_from_pdf_skill(db).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invoice_site_pos_skill_export_parses() {
        let export: SkillExport =
            serde_json::from_str(INVOICE_SITE_POS_SKILL).expect("index.json");
        assert_eq!(export.name, "invoice-site-purchase-orders");
        assert!(export.content.contains("create_invoices_for_site"));
        assert!(export.content.contains("search_products"));
        assert!(export.content.contains("link_site_invoice"));
        assert!(export.content.contains("description"));
        assert!(export.content.contains("return objects"));
        assert!(!export.content.contains("JSON string"));
        assert!(export.description.contains("purchase order"));
    }

    #[test]
    fn create_po_from_pdf_skill_export_parses() {
        let export: SkillExport =
            serde_json::from_str(CREATE_PO_FROM_PDF_SKILL).expect("index.json");
        assert_eq!(export.name, "create-purchase-order-from-pdf");
        assert!(export.content.contains("create_purchase_order_from_pdf"));
        assert!(export.content.contains("find_site"));
        assert!(export.content.contains("file_id"));
        assert!(export.content.contains("dry_run"));
        assert!(export.description.contains("PDF"));
    }
}
