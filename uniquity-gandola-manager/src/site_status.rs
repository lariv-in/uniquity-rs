use std::fmt;
use std::str::FromStr;

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

pub const SITE_STATUS_STARTED: &str = "started";
pub const SITE_STATUS_DOCS_DONE: &str = "docs_done";
pub const SITE_STATUS_COMPLETED: &str = "completed";
pub const SITE_STATUS_PAYMENT_SETTLED: &str = "payment_settled";

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(32))")]
pub enum SiteStatus {
    #[default]
    #[sea_orm(string_value = "started")]
    Started,
    #[sea_orm(string_value = "docs_done")]
    DocsDone,
    #[sea_orm(string_value = "completed")]
    Completed,
    #[sea_orm(string_value = "payment_settled")]
    PaymentSettled,
}

impl SiteStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Started => SITE_STATUS_STARTED,
            Self::DocsDone => SITE_STATUS_DOCS_DONE,
            Self::Completed => SITE_STATUS_COMPLETED,
            Self::PaymentSettled => SITE_STATUS_PAYMENT_SETTLED,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Started => "Started",
            Self::DocsDone => "Docs Done",
            Self::Completed => "Completed",
            Self::PaymentSettled => "Payment Settled",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            SITE_STATUS_STARTED => Some(Self::Started),
            SITE_STATUS_DOCS_DONE => Some(Self::DocsDone),
            SITE_STATUS_COMPLETED => Some(Self::Completed),
            SITE_STATUS_PAYMENT_SETTLED => Some(Self::PaymentSettled),
            _ => None,
        }
    }

    pub fn choices() -> &'static [(&'static str, &'static str)] {
        &[
            (SITE_STATUS_STARTED, "Started"),
            (SITE_STATUS_DOCS_DONE, "Docs Done"),
            (SITE_STATUS_COMPLETED, "Completed"),
            (SITE_STATUS_PAYMENT_SETTLED, "Payment Settled"),
        ]
    }

    pub fn badge_class(self) -> &'static str {
        match self {
            Self::Started => "badge badge-warning",
            Self::DocsDone => "badge badge-primary",
            Self::Completed => "badge badge-info",
            Self::PaymentSettled => "badge badge-success",
        }
    }
}

impl fmt::Display for SiteStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl FromStr for SiteStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or_else(|| format!("invalid SiteStatus: {s:?}"))
    }
}

impl From<SiteStatus> for String {
    fn from(v: SiteStatus) -> Self {
        v.as_str().into()
    }
}
