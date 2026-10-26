//! Column filters for the invoices table on a site detail page.
//!
//! Semantics match the invoice hub: text columns are case-insensitive
//! substrings, dates and amounts are inclusive ranges, and a product keeps
//! invoices that have a line for it. The list is already limited to the site.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use std::str::FromStr;

use lariv_core::datetime::{
    DatetimeLocalInput, parse_date, parse_date_start_in_tz, parse_naive_datetime,
};

use crate::scope::SiteInvoiceRow;

/// Query params for the site invoices filter. HTML names match
/// [`crate::forms::SiteInvoiceFilterForm`]. `InvoiceNumber` is not `Number`
/// because the purchase-order table on the same page already uses `Number`.
#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct SiteInvoiceFilterQuery {
    #[serde(default, rename = "ID")]
    pub id: Option<String>,
    #[serde(default, rename = "InvoiceNumber")]
    pub number: Option<String>,
    #[serde(default, rename = "InvoiceStatus")]
    pub status: Option<String>,
    #[serde(default, rename = "Customer")]
    pub customer: Option<String>,
    #[serde(default, rename = "OpenBalanceMin")]
    pub open_balance_min: Option<String>,
    #[serde(default, rename = "OpenBalanceMax")]
    pub open_balance_max: Option<String>,
    #[serde(default, rename = "DatetimeFrom")]
    pub datetime_from: Option<String>,
    #[serde(default, rename = "DatetimeTo")]
    pub datetime_to: Option<String>,
    #[serde(default, rename = "DeliveryDateFrom")]
    pub delivery_date_from: Option<String>,
    #[serde(default, rename = "DeliveryDateTo")]
    pub delivery_date_to: Option<String>,
    #[serde(default, rename = "UntaxedMin")]
    pub untaxed_min: Option<String>,
    #[serde(default, rename = "UntaxedMax")]
    pub untaxed_max: Option<String>,
    #[serde(default, rename = "TotalMin")]
    pub total_min: Option<String>,
    #[serde(default, rename = "TotalMax")]
    pub total_max: Option<String>,
    #[serde(default, rename = "TaxMin")]
    pub tax_min: Option<String>,
    #[serde(default, rename = "TaxMax")]
    pub tax_max: Option<String>,
    #[serde(default, rename = "ProductID")]
    pub product_id: Option<String>,
    #[serde(default, rename = "ProductCountMin")]
    pub product_count_min: Option<String>,
    #[serde(default, rename = "ProductCountMax")]
    pub product_count_max: Option<String>,
    #[serde(default, rename = "FinalDueFrom")]
    pub final_due_from: Option<String>,
    #[serde(default, rename = "FinalDueTo")]
    pub final_due_to: Option<String>,
}

impl SiteInvoiceFilterQuery {
    pub fn text(&self, raw: &Option<String>) -> String {
        raw.clone().unwrap_or_default()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Bound<T> {
    Any,
    Reject,
    Value(T),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DateTimeEnd {
    Inclusive(DateTime<Utc>),
    Exclusive(DateTime<Utc>),
}

struct ParsedFilters {
    id: Option<String>,
    number: Option<String>,
    status: Option<String>,
    customer: Option<String>,
    open_balance_min: Bound<Decimal>,
    open_balance_max: Bound<Decimal>,
    datetime_from: Bound<DateTime<Utc>>,
    datetime_to: Bound<DateTimeEnd>,
    delivery_from: Bound<NaiveDate>,
    delivery_to: Bound<NaiveDate>,
    untaxed_min: Bound<Decimal>,
    untaxed_max: Bound<Decimal>,
    total_min: Bound<Decimal>,
    total_max: Bound<Decimal>,
    tax_min: Bound<Decimal>,
    tax_max: Bound<Decimal>,
    product_id: Bound<i64>,
    product_min: Bound<i64>,
    product_max: Bound<i64>,
    final_due_from: Bound<NaiveDate>,
    final_due_to: Bound<NaiveDate>,
}

impl ParsedFilters {
    fn from_query(input: &SiteInvoiceFilterQuery, tz: &str) -> Self {
        Self {
            id: needle(&input.id),
            number: needle(&input.number),
            status: needle(&input.status),
            customer: needle(&input.customer),
            open_balance_min: parse_decimal(&input.open_balance_min),
            open_balance_max: parse_decimal(&input.open_balance_max),
            datetime_from: parse_datetime_start(&input.datetime_from, tz),
            datetime_to: parse_datetime_end(&input.datetime_to, tz),
            delivery_from: parse_date_bound(&input.delivery_date_from),
            delivery_to: parse_date_bound(&input.delivery_date_to),
            untaxed_min: parse_decimal(&input.untaxed_min),
            untaxed_max: parse_decimal(&input.untaxed_max),
            total_min: parse_decimal(&input.total_min),
            total_max: parse_decimal(&input.total_max),
            tax_min: parse_decimal(&input.tax_min),
            tax_max: parse_decimal(&input.tax_max),
            product_id: parse_i64(&input.product_id),
            product_min: parse_i64(&input.product_count_min),
            product_max: parse_i64(&input.product_count_max),
            final_due_from: parse_date_bound(&input.final_due_from),
            final_due_to: parse_date_bound(&input.final_due_to),
        }
    }

    fn rejected(&self) -> bool {
        bound_rejected(self.open_balance_min)
            || bound_rejected(self.open_balance_max)
            || bound_rejected(self.datetime_from)
            || bound_rejected(self.datetime_to)
            || bound_rejected(self.delivery_from)
            || bound_rejected(self.delivery_to)
            || bound_rejected(self.untaxed_min)
            || bound_rejected(self.untaxed_max)
            || bound_rejected(self.total_min)
            || bound_rejected(self.total_max)
            || bound_rejected(self.tax_min)
            || bound_rejected(self.tax_max)
            || bound_rejected(self.product_id)
            || bound_rejected(self.product_min)
            || bound_rejected(self.product_max)
            || bound_rejected(self.final_due_from)
            || bound_rejected(self.final_due_to)
    }

    fn matches(&self, row: &SiteInvoiceRow) -> bool {
        contains_ci(self.id.as_deref(), &row.display_id.to_string())
            && contains_ci(self.number.as_deref(), &row.number)
            && status_matches(self.status.as_deref(), &row.status)
            && contains_ci(self.customer.as_deref(), &row.customer)
            && decimal_opt_in_range(
                self.open_balance_min,
                self.open_balance_max,
                row.open_balance_amount,
            )
            && datetime_in_range(self.datetime_from, self.datetime_to, row.sort_datetime)
            && date_in_range(self.delivery_from, self.delivery_to, row.delivery_on)
            && decimal_in_range(self.untaxed_min, self.untaxed_max, row.untaxed)
            && decimal_in_range(self.total_min, self.total_max, row.total)
            && decimal_in_range(self.tax_min, self.tax_max, row.tax)
            && product_matches(self.product_id, &row.product_ids)
            && i64_in_range(
                self.product_min,
                self.product_max,
                i64::from(row.product_count_n),
            )
            && date_in_range(self.final_due_from, self.final_due_to, row.final_due)
    }
}

pub fn invoice_row_matches(
    row: &SiteInvoiceRow,
    filters: &SiteInvoiceFilterQuery,
    tz: &str,
) -> bool {
    let parsed = ParsedFilters::from_query(filters, tz);
    !parsed.rejected() && parsed.matches(row)
}

pub fn invoice_filters_reject_all(filters: &SiteInvoiceFilterQuery, tz: &str) -> bool {
    ParsedFilters::from_query(filters, tz).rejected()
}

fn contains_ci(needle: Option<&str>, haystack: &str) -> bool {
    match needle {
        None => true,
        Some(needle) => haystack.to_lowercase().contains(&needle.to_lowercase()),
    }
}

fn status_matches(filter: Option<&str>, status: &str) -> bool {
    match filter {
        None => true,
        Some(filter) => status.eq_ignore_ascii_case(filter),
    }
}

fn product_matches(product_id: Bound<i64>, ids: &[i64]) -> bool {
    match product_id {
        Bound::Any => true,
        Bound::Reject => false,
        Bound::Value(id) => ids.contains(&id),
    }
}

fn nonempty(raw: &Option<String>) -> Option<&str> {
    raw.as_deref().map(str::trim).filter(|s| !s.is_empty())
}

fn needle(raw: &Option<String>) -> Option<String> {
    nonempty(raw).map(str::to_string)
}

fn bound_rejected<T>(bound: Bound<T>) -> bool {
    matches!(bound, Bound::Reject)
}

fn parse_decimal(raw: &Option<String>) -> Bound<Decimal> {
    let Some(s) = nonempty(raw) else {
        return Bound::Any;
    };
    match Decimal::from_str(&s.replace(',', "")) {
        Ok(value) => Bound::Value(value),
        Err(_) => Bound::Reject,
    }
}

fn parse_i64(raw: &Option<String>) -> Bound<i64> {
    let Some(s) = nonempty(raw) else {
        return Bound::Any;
    };
    match s.parse() {
        Ok(value) => Bound::Value(value),
        Err(_) => Bound::Reject,
    }
}

fn parse_date_bound(raw: &Option<String>) -> Bound<NaiveDate> {
    let Some(s) = nonempty(raw) else {
        return Bound::Any;
    };
    match parse_date(s) {
        Some(date) => Bound::Value(date),
        None => Bound::Reject,
    }
}

fn parse_datetime_start(raw: &Option<String>, tz: &str) -> Bound<DateTime<Utc>> {
    let Some(s) = nonempty(raw) else {
        return Bound::Any;
    };
    if let Some(dt) = DatetimeLocalInput::from_raw(s).to_stored(tz) {
        return Bound::Value(dt);
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Bound::Value(dt.with_timezone(&Utc));
    }
    match parse_date_start_in_tz(s, tz) {
        Some(dt) => Bound::Value(dt),
        None => Bound::Reject,
    }
}

fn parse_datetime_end(raw: &Option<String>, tz: &str) -> Bound<DateTimeEnd> {
    let Some(s) = nonempty(raw) else {
        return Bound::Any;
    };
    if parse_naive_datetime(s).is_some() {
        return match DatetimeLocalInput::from_raw(s).to_stored(tz) {
            Some(dt) => Bound::Value(DateTimeEnd::Inclusive(dt)),
            None => Bound::Reject,
        };
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Bound::Value(DateTimeEnd::Inclusive(dt.with_timezone(&Utc)));
    }
    let Some(date) = parse_date(s) else {
        return Bound::Reject;
    };
    let Some(next) = date.succ_opt() else {
        return Bound::Reject;
    };
    let label = next.format("%d/%m/%Y").to_string();
    match parse_date_start_in_tz(&label, tz) {
        Some(dt) => Bound::Value(DateTimeEnd::Exclusive(dt)),
        None => Bound::Reject,
    }
}

fn decimal_in_range(min: Bound<Decimal>, max: Bound<Decimal>, value: Decimal) -> bool {
    decimal_opt_in_range(min, max, Some(value))
}

fn decimal_opt_in_range(min: Bound<Decimal>, max: Bound<Decimal>, value: Option<Decimal>) -> bool {
    match min {
        Bound::Any => {}
        Bound::Reject => return false,
        Bound::Value(bound) => match value {
            Some(value) if value >= bound => {}
            _ => return false,
        },
    }
    match max {
        Bound::Any => {}
        Bound::Reject => return false,
        Bound::Value(bound) => match value {
            Some(value) if value <= bound => {}
            _ => return false,
        },
    }
    true
}

fn i64_in_range(min: Bound<i64>, max: Bound<i64>, value: i64) -> bool {
    match min {
        Bound::Any => {}
        Bound::Reject => return false,
        Bound::Value(bound) if value < bound => return false,
        Bound::Value(_) => {}
    }
    match max {
        Bound::Any => {}
        Bound::Reject => return false,
        Bound::Value(bound) if value > bound => return false,
        Bound::Value(_) => {}
    }
    true
}

fn date_in_range(min: Bound<NaiveDate>, max: Bound<NaiveDate>, value: Option<NaiveDate>) -> bool {
    match min {
        Bound::Any => {}
        Bound::Reject => return false,
        Bound::Value(bound) => match value {
            Some(date) if date >= bound => {}
            _ => return false,
        },
    }
    match max {
        Bound::Any => {}
        Bound::Reject => return false,
        Bound::Value(bound) => match value {
            Some(date) if date <= bound => {}
            _ => return false,
        },
    }
    true
}

fn datetime_in_range(
    from: Bound<DateTime<Utc>>,
    to: Bound<DateTimeEnd>,
    value: Option<DateTime<Utc>>,
) -> bool {
    match from {
        Bound::Any => {}
        Bound::Reject => return false,
        Bound::Value(start) => match value {
            Some(dt) if dt >= start => {}
            _ => return false,
        },
    }
    match to {
        Bound::Any => {}
        Bound::Reject => return false,
        Bound::Value(DateTimeEnd::Inclusive(end)) => match value {
            Some(dt) if dt <= end => {}
            _ => return false,
        },
        Bound::Value(DateTimeEnd::Exclusive(end)) => match value {
            Some(dt) if dt < end => {}
            _ => return false,
        },
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use rust_decimal::Decimal;

    fn row() -> SiteInvoiceRow {
        SiteInvoiceRow {
            display_id: 42,
            draft_id: 7,
            number: "INV-100".into(),
            href: String::new(),
            status: "Posted".into(),
            customer: "Acme Co".into(),
            open_balance: "10.00".into(),
            open_balance_amount: Some(Decimal::from(10)),
            date: String::new(),
            sort_datetime: Some(Utc.with_ymd_and_hms(2026, 4, 1, 15, 0, 0).unwrap()),
            delivery_date: String::new(),
            delivery_on: Some(NaiveDate::from_ymd_opt(2026, 4, 2).unwrap()),
            untaxed_amount: String::new(),
            untaxed: Decimal::from(100),
            total_amount: String::new(),
            total: Decimal::from(118),
            tax_levied: String::new(),
            tax: Decimal::from(18),
            product_count: "2".into(),
            product_count_n: 2,
            product_ids: vec![5, 9],
            final_due_date: String::new(),
            final_due: Some(NaiveDate::from_ymd_opt(2026, 5, 1).unwrap()),
        }
    }

    fn filters() -> SiteInvoiceFilterQuery {
        SiteInvoiceFilterQuery::default()
    }

    #[test]
    fn text_filters_are_case_insensitive_substrings() {
        let mut q = filters();
        q.id = Some("4".into());
        q.number = Some("inv-1".into());
        q.customer = Some("ACME".into());
        q.status = Some("posted".into());
        assert!(invoice_row_matches(&row(), &q, "UTC"));
        q.number = Some("missing".into());
        assert!(!invoice_row_matches(&row(), &q, "UTC"));
    }

    #[test]
    fn date_only_range_covers_the_whole_local_day() {
        let mut q = filters();
        q.datetime_from = Some("01/04/2026".into());
        q.datetime_to = Some("01/04/2026".into());
        assert!(invoice_row_matches(&row(), &q, "UTC"));
        q.datetime_to = Some("01/04/2026 10:00:00".into());
        assert!(!invoice_row_matches(&row(), &q, "UTC"));
    }

    #[test]
    fn amount_and_count_ranges_are_inclusive() {
        let mut q = filters();
        q.open_balance_min = Some("10".into());
        q.open_balance_max = Some("10".into());
        q.untaxed_min = Some("100".into());
        q.total_max = Some("118".into());
        q.tax_min = Some("18".into());
        q.product_count_min = Some("2".into());
        q.product_count_max = Some("2".into());
        assert!(invoice_row_matches(&row(), &q, "UTC"));
        q.total_max = Some("100".into());
        assert!(!invoice_row_matches(&row(), &q, "UTC"));
    }

    #[test]
    fn open_balance_filter_excludes_rows_without_one() {
        let mut draft = row();
        draft.open_balance_amount = None;
        let mut q = filters();
        q.open_balance_min = Some("0".into());
        assert!(!invoice_row_matches(&draft, &q, "UTC"));
        q.open_balance_min = None;
        assert!(invoice_row_matches(&draft, &q, "UTC"));
    }

    #[test]
    fn product_filter_matches_a_line() {
        let mut q = filters();
        q.product_id = Some("9".into());
        assert!(invoice_row_matches(&row(), &q, "UTC"));
        q.product_id = Some("3".into());
        assert!(!invoice_row_matches(&row(), &q, "UTC"));
    }

    #[test]
    fn unparseable_amount_rejects_every_row() {
        let mut q = filters();
        q.total_min = Some("nope".into());
        assert!(invoice_filters_reject_all(&q, "UTC"));
        assert!(!invoice_row_matches(&row(), &q, "UTC"));
    }

    #[test]
    fn delivery_and_due_dates_use_inclusive_ranges() {
        let mut q = filters();
        q.delivery_date_from = Some("02/04/2026".into());
        q.delivery_date_to = Some("02/04/2026".into());
        q.final_due_from = Some("01/05/2026".into());
        q.final_due_to = Some("01/05/2026".into());
        assert!(invoice_row_matches(&row(), &q, "UTC"));
        q.final_due_to = Some("30/04/2026".into());
        assert!(!invoice_row_matches(&row(), &q, "UTC"));
    }
}
