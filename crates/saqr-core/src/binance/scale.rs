use crate::core::unified_market_state::SCALE_8;

pub fn decimal_str_to_scaled(value: &str) -> Result<i64, &'static str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("empty decimal");
    }
    let parts: Vec<&str> = trimmed.split('.').collect();
    match parts.as_slice() {
        [whole] => {
            let w: i64 = whole.parse().map_err(|_| "parse whole")?;
            Ok(w.saturating_mul(SCALE_8))
        }
        [whole, frac] => {
            let w: i64 = whole.parse().map_err(|_| "parse whole")?;
            let mut frac_str = frac.to_string();
            if frac_str.len() > 8 {
                frac_str.truncate(8);
            }
            while frac_str.len() < 8 {
                frac_str.push('0');
            }
            let f: i64 = frac_str.parse().map_err(|_| "parse frac")?;
            let sign = if w < 0 { -1 } else { 1 };
            Ok(w.saturating_mul(SCALE_8) + sign * f.abs())
        }
        _ => Err("invalid decimal"),
    }
}

pub fn scaled_to_decimal_str(scaled: i64) -> String {
    let sign = if scaled < 0 { "-" } else { "" };
    let abs = scaled.abs();
    let whole = abs / SCALE_8;
    let frac = abs % SCALE_8;
    format!("{sign}{whole}.{frac:08}")
}
