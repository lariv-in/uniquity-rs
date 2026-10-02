//! Deployment role `hr`: Accounting and Gandola Manager, plus the routes they need.

use frunk::{HCons, hlist::HList};
use sea_orm::DatabaseConnection;

use lariv_rs::{
    app::{App, MountedApp},
    apps::{AppsCapability, AppsRegistrar},
    capability::CapStore,
    db::{DbCap, DbTag},
    hooks::{AttachState, RunSeed},
    plugins::{
        customer::routes::{CustomerMutate, CustomerView},
        finance_accounts::{
            ACCOUNTING_APP_KEY,
            routes::{FinanceAccountsMutate, FinanceAccountsView},
        },
        finance_creditnotes::routes::FinanceCreditNotesView,
        finance_invoices::routes::{FinanceInvoicesMutate, FinanceInvoicesView},
        finance_products::routes::{FinanceProductsMutate, FinanceProductsView},
        finance_taxes::routes::{FinanceTaxesMutate, FinanceTaxesView},
        users::role_authorization::{RoleAuthorizationRegistrar, RoleAuthorizationRegistry},
    },
    traits::{
        add::{AddCapability, CapTagAbsent},
        get::{GetByCapTag, GetByTag},
    },
};

pub const HR_ROLE: &str = "hr";

pub struct HrRoleTag;

#[derive(Clone)]
pub struct HrRoleState {
    pub db: DatabaseConnection,
}

impl HrRoleState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

lariv_rs::define_passthrough_cap!(HrRoleStateCap, HrRoleTag, HrRoleState);

lariv_rs::define_plugin_install! {
    plugin: HrRoleTag;
    steps: [
        apps(AppsHook),
        cap_hook(lariv_rs::plugins::users::role_authorization::RoleAuthorizationTag, lariv_rs::plugins::users::role_authorization::RoleAuthorizationCap, RoleHook),
        state(StateHook),
        seeds(SeedsHook),
    ]
}

#[derive(Clone, Copy, Default)]
pub struct AppsHook;

impl AppsRegistrar for AppsHook {
    fn register_apps(self, apps: AppsCapability) -> AppsCapability {
        let Some(mut accounting) = apps
            .apps()
            .iter()
            .find(|tile| tile.key == ACCOUNTING_APP_KEY)
            .cloned()
        else {
            return apps;
        };
        if !accounting.roles.iter().any(|role| role == HR_ROLE) {
            accounting.roles.push(HR_ROLE.into());
        }
        apps.register(accounting)
    }
}

#[derive(Clone, Copy, Default)]
pub struct RoleHook;

impl RoleAuthorizationRegistrar for RoleHook {
    fn register_roles(self, registry: RoleAuthorizationRegistry) -> RoleAuthorizationRegistry {
        registry
            .patch::<FinanceAccountsView>(allow_hr)
            .patch::<FinanceAccountsMutate>(allow_hr)
            // FinanceAccountsPreferencesMutate stays superuser-only.
            .patch::<FinanceInvoicesView>(allow_hr)
            .patch::<FinanceInvoicesMutate>(allow_hr)
            .patch::<FinanceProductsView>(allow_hr)
            .patch::<FinanceProductsMutate>(allow_hr)
            .patch::<FinanceTaxesView>(allow_hr)
            .patch::<FinanceTaxesMutate>(allow_hr)
            .patch::<FinanceCreditNotesView>(allow_hr)
            .patch::<CustomerView>(allow_hr)
            .patch::<CustomerMutate>(allow_hr)
    }
}

fn allow_hr(roles: &mut Vec<String>) {
    if !roles.iter().any(|role| role == HR_ROLE) {
        roles.push(HR_ROLE.into());
    }
}

#[derive(Clone, Copy, Default)]
pub struct StateHook;

impl<L, DbIdx, TagProof> AttachState<L, (DbIdx, TagProof)> for StateHook
where
    L: GetByCapTag<DbTag, DbIdx, Value = DbCap>,
    L: HList + CapTagAbsent<HrRoleTag, TagProof>,
{
    type Output = HCons<HrRoleStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        app.add_capability(CapStore::with_items(HrRoleState::new(conn)))
    }
}

#[derive(Clone, Copy, Default)]
pub struct SeedsHook;

#[async_trait::async_trait]
impl<M, Idx> RunSeed<M, Idx> for SeedsHook
where
    M: GetByTag<HrRoleTag, Idx, Value = HrRoleState> + Sync,
{
    async fn run_seed(app: &MountedApp<M>) -> anyhow::Result<()> {
        seed(app.get_capability_output::<HrRoleTag, Idx>()).await?;
        Ok(())
    }
}

async fn seed(state: &HrRoleState) -> anyhow::Result<()> {
    use chrono::Utc;
    use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};

    use lariv_rs::plugins::users::entities::role::{self, Entity as RoleEntity};

    if RoleEntity::find()
        .filter(role::Column::Name.eq(HR_ROLE))
        .one(&state.db)
        .await?
        .is_some()
    {
        sync_roles_id_sequence(&state.db).await?;
        return Ok(());
    }

    // Roles such as `unassigned` are inserted with an explicit id, which does not
    // advance the Postgres sequence. Using that sequence collides on `roles_pkey`.
    let next_id = RoleEntity::find()
        .all(&state.db)
        .await?
        .into_iter()
        .map(|role| role.id)
        .max()
        .unwrap_or(0)
        + 1;
    let now = Utc::now();
    let model = role::ActiveModel {
        id: Set(next_id),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        name: Set(HR_ROLE.into()),
        title: Set("HR".into()),
        description: Set("Accounting and Gandola Manager.".into()),
    };
    match model.insert(&state.db).await {
        Ok(_) => {}
        Err(err) => {
            if RoleEntity::find()
                .filter(role::Column::Name.eq(HR_ROLE))
                .one(&state.db)
                .await?
                .is_none()
            {
                return Err(err.into());
            }
        }
    }
    sync_roles_id_sequence(&state.db).await?;
    Ok(())
}

async fn sync_roles_id_sequence(db: &sea_orm::DatabaseConnection) -> anyhow::Result<()> {
    use sea_orm::ConnectionTrait;

    db.execute_unprepared(
        "SELECT setval(pg_get_serial_sequence('roles', 'id'), (SELECT COALESCE(MAX(id), 1) FROM roles))",
    )
    .await?;
    Ok(())
}
