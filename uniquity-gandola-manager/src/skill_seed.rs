//! Idempotent seed for the site purchase-order invoicing skill.

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

use lariv_rs::plugins::llm_assistant::entities::skill::{self, Entity as SkillEntity};

const SKILL_EXPORT: &str = include_str!("../skills/invoice-site-purchase-orders/index.json");

#[derive(serde::Deserialize)]
struct SkillExport {
    name: String,
    description: String,
    content: String,
}

pub async fn ensure_invoice_site_pos_skill(db: &DatabaseConnection) -> anyhow::Result<()> {
    let export: SkillExport = serde_json::from_str(SKILL_EXPORT)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_skill_export_parses() {
        let export: SkillExport = serde_json::from_str(SKILL_EXPORT).expect("index.json");
        assert_eq!(export.name, "invoice-site-purchase-orders");
        assert!(export.content.contains("create_invoices_for_site"));
        assert!(export.content.contains("search_products"));
        assert!(export.content.contains("link_site_invoice"));
        assert!(export.content.contains("description"));
        assert!(export.content.contains("return objects"));
        assert!(!export.content.contains("JSON string"));
        assert!(export.description.contains("purchase order"));
    }
}
