use chrono::NaiveDate;

use crate::entities::site;

/// First linked site whose date window includes `today` (Odoo `_compute_current_site`).
pub fn current_site_for<'a>(sites: &'a [site::Model], today: NaiveDate) -> Option<&'a site::Model> {
    for site in sites {
        if site_is_current(site, today) {
            return Some(site);
        }
    }
    None
}

pub fn site_is_current(site: &site::Model, today: NaiveDate) -> bool {
    match (site.start_date, site.end_date) {
        (Some(start), Some(end)) => start <= today && today <= end,
        (Some(start), None) => start <= today,
        (None, Some(end)) => today <= end,
        (None, None) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::site_status::SiteStatus;

    fn site(
        id: i64,
        start: Option<(i32, u32, u32)>,
        end: Option<(i32, u32, u32)>,
    ) -> site::Model {
        fn d(y: i32, m: u32, day: u32) -> NaiveDate {
            NaiveDate::from_ymd_opt(y, m, day).unwrap()
        }
        site::Model {
            id,
            created_at: None,
            updated_at: None,
            name: format!("Site {id}"),
            address: None,
            start_date: start.map(|(y, m, day)| d(y, m, day)),
            end_date: end.map(|(y, m, day)| d(y, m, day)),
            customer_id: 1,
            status: SiteStatus::Started,
            po_rent: None,
            po_dti: None,
            po_tpi: None,
            po_extn1: None,
            po_extn2: None,
            po_extn3: None,
        }
    }

    #[test]
    fn both_dates_inclusive() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 14).unwrap();
        let sites = vec![
            site(1, Some((2026, 1, 1)), Some((2026, 6, 30))),
            site(2, Some((2026, 8, 1)), Some((2026, 8, 31))),
        ];
        assert_eq!(current_site_for(&sites, today).map(|s| s.id), Some(2));
    }

    #[test]
    fn start_only_open_ended() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 14).unwrap();
        let sites = vec![site(1, Some((2026, 8, 1)), None)];
        assert_eq!(current_site_for(&sites, today).map(|s| s.id), Some(1));
    }

    #[test]
    fn neither_date_never_matches() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 14).unwrap();
        let sites = vec![site(1, None, None)];
        assert!(current_site_for(&sites, today).is_none());
    }

    #[test]
    fn first_match_wins() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 14).unwrap();
        let sites = vec![
            site(1, Some((2026, 8, 1)), Some((2026, 8, 31))),
            site(2, Some((2026, 8, 1)), Some((2026, 8, 31))),
        ];
        assert_eq!(current_site_for(&sites, today).map(|s| s.id), Some(1));
    }
}
