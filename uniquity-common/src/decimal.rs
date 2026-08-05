//! Six-decimal finance arithmetic (mirrors Go fields.DecimalSix).

use rust_decimal::Decimal;

pub fn normalize(d: Decimal) -> Decimal {
    d.round_dp(6).normalize()
}

pub fn dec_mul(a: Decimal, b: Decimal) -> Decimal {
    normalize(a * b)
}

pub fn dec_sum(a: Decimal, b: Decimal) -> Decimal {
    normalize(a + b)
}

pub fn dec_neg(a: Decimal) -> Decimal {
    normalize(-a)
}

pub fn dec_sub(a: Decimal, b: Decimal) -> Decimal {
    normalize(a - b)
}

pub fn dec_abs(a: Decimal) -> Decimal {
    normalize(a.abs())
}

pub fn dec_is_zero(d: Decimal) -> bool {
    d.is_zero()
}

pub fn dec_cmp(a: Decimal, b: Decimal) -> std::cmp::Ordering {
    a.cmp(&b)
}

pub fn decimal_display(d: Decimal) -> String {
    let s = normalize(d).to_string();
    if !s.contains('.') {
        return s;
    }
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s.is_empty() {
        "0".to_string()
    } else {
        s.to_string()
    }
}

pub fn decimal_display_withholding(d: Decimal) -> String {
    if dec_is_zero(d) {
        "—".to_string()
    } else {
        format!("({})", decimal_display(d))
    }
}

pub fn parse_decimal(s: &str) -> Option<Decimal> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    s.parse().ok().map(normalize)
}

pub fn optional_u64(v: Option<i64>) -> u64 {
    v.unwrap_or(0).max(0) as u64
}
