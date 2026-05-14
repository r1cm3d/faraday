//! Panel submodules and shared byte-decode and formatting utilities.
pub mod adas;
pub mod analytics;
pub mod body;
pub mod climate;
pub mod engine;
pub mod glossary;
pub mod health;
pub mod infotainment;
pub mod safety;
pub mod transmission;

pub use adas::AdasPanel;
pub use analytics::AnalyticsPanel;
pub use body::BodyPanel;
pub use climate::ClimatePanel;
pub use engine::EnginePanel;
pub use glossary::GlossaryPanel;
pub use health::HealthPanel;
pub use infotainment::InfotainmentPanel;
pub use safety::SafetyPanel;
pub use transmission::TransmissionPanel;

/// Raw-byte temperature offset used by Ford ECUs (subtract to get °C).
pub const TEMP_OFFSET_CELSIUS: f64 = 40.0;
/// Maximum PWM duty cycle value for 0-100% scaling.
pub const PWM_MAX: f64 = 255.0;
/// Conversion factor from PSI to kPa (1 PSI = 6.89476 kPa).
pub const KPA_PER_PSI: f64 = 6.89476;
/// Rolling history window length for fluid temperature and fuel-rate buffers.
pub const HISTORY_LEN: usize = 60;
/// Sentinel value used by the PAM to signal an absent/out-of-range sensor.
pub const SENSOR_SENTINEL: u16 = 0xFFFF;

/// Assemble a big-endian `u16` from a high byte and a low byte.
pub fn u16_be(high: u8, low: u8) -> u16 {
    ((high as u16) << 8) | low as u16
}

/// Convert a Ford raw-byte temperature to degrees Celsius (subtract 40° offset).
pub fn celsius_from_raw(raw: u8) -> f64 {
    raw as f64 - TEMP_OFFSET_CELSIUS
}

/// Convert a 0-255 PWM duty cycle byte to a 0-100 percent value.
pub fn pwm_to_percent(pwm: u8) -> f64 {
    pwm as f64 * 100.0 / PWM_MAX
}

/// Map a raw sensor `u16` to `None` when it equals `SENSOR_SENTINEL`, otherwise `Some`.
pub fn sensor_opt(val: u16) -> Option<u16> {
    if val == SENSOR_SENTINEL {
        None
    } else {
        Some(val)
    }
}

/// Format an optional `f64` with the given decimal precision and unit suffix, or `"--"`.
pub fn fmt_opt_f(val: Option<f64>, precision: usize, unit: &str) -> String {
    match val {
        Some(v) => format!("{:.prec$}{}", v, unit, prec = precision),
        None => "--".to_string(),
    }
}

/// Format an optional temperature as `"°C / °F"` at the given precision, or `"--"`.
pub fn fmt_temp(val: Option<f64>, precision: usize) -> String {
    match val {
        Some(c) => {
            let f = c * 9.0 / 5.0 + 32.0;
            format!("{:.prec$}°C / {:.prec$}°F", c, f, prec = precision)
        }
        None => "--".to_string(),
    }
}

/// Format an optional speed in km/h alongside its mph equivalent, or `"--"`.
pub fn fmt_speed_kmh(val: Option<f64>) -> String {
    match val {
        Some(kmh) => format!("{:.0} km/h / {:.0} mph", kmh, kmh * 0.621371),
        None => "--".to_string(),
    }
}

/// Format an optional pressure in kPa alongside its PSI equivalent, or `"--"`.
pub fn fmt_pressure_kpa(val: Option<f64>) -> String {
    match val {
        Some(kpa) => format!("{:.0} kPa / {:.1} PSI", kpa, kpa / 6.895),
        None => "--".to_string(),
    }
}

/// Format an optional TPMS pressure in kPa, PSI, and bar, or `"--"`.
pub fn fmt_tpms_kpa(val: Option<f64>) -> String {
    match val {
        Some(kpa) => format!(
            "{:.0} kPa / {:.0} PSI / {:.2} bar",
            kpa,
            kpa / 6.895,
            kpa / 100.0
        ),
        None => "--".to_string(),
    }
}

/// Format an optional distance in cm alongside its inch equivalent, or `"--"`.
pub fn fmt_dist_cm(val: Option<u16>) -> String {
    match val {
        Some(cm) => format!("{}cm / {:.1}in", cm, cm as f64 / 2.54),
        None => "--".to_string(),
    }
}

/// Format an optional fuel rate in L/h alongside its gal/h equivalent, or `"--"`.
pub fn fmt_fuel_rate(val: Option<f64>) -> String {
    match val {
        Some(lph) => format!("{:.2} L/h / {:.2} gal/h", lph, lph * 0.264172),
        None => "--".to_string(),
    }
}

/// Format fuel economy in km/L, L/100 km, and MPG, or `"--"` if data is unavailable.
pub fn fmt_fuel_economy(speed: Option<f64>, fuel_rate: Option<f64>) -> String {
    match (speed, fuel_rate) {
        (Some(kmh), Some(lph)) if kmh >= 1.0 && lph > 0.0 => {
            let kml = kmh / lph;
            let l100 = 100.0 / kml;
            let mpg = kml * 2.35215;
            format!("{:.1} km/L / {:.1} L/100km / {:.1} MPG", kml, l100, mpg)
        }
        _ => "--".to_string(),
    }
}

/// Clamp `val` to `[0, max]` and return the result as an integer percentage.
pub fn clamp_percent(val: f64, max: f64) -> u16 {
    ((val / max * 100.0).clamp(0.0, 100.0)) as u16
}
