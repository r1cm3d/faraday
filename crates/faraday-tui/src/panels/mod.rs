pub mod adas;
pub mod analytics;
pub mod body;
pub mod climate;
pub mod engine;
pub mod health;
pub mod infotainment;
pub mod safety;
pub mod transmission;

pub use adas::AdasPanel;
pub use analytics::AnalyticsPanel;
pub use body::BodyPanel;
pub use climate::ClimatePanel;
pub use engine::EnginePanel;
pub use health::HealthPanel;
pub use infotainment::InfotainmentPanel;
pub use safety::SafetyPanel;
pub use transmission::TransmissionPanel;

pub fn fmt_opt_f(val: Option<f64>, precision: usize, unit: &str) -> String {
    match val {
        Some(v) => format!("{:.prec$}{}", v, unit, prec = precision),
        None => "--".to_string(),
    }
}

pub fn fmt_temp(val: Option<f64>, precision: usize) -> String {
    match val {
        Some(c) => {
            let f = c * 9.0 / 5.0 + 32.0;
            format!("{:.prec$}°C / {:.prec$}°F", c, f, prec = precision)
        }
        None => "--".to_string(),
    }
}

pub fn fmt_speed_kmh(val: Option<f64>) -> String {
    match val {
        Some(kmh) => format!("{:.0} km/h / {:.0} mph", kmh, kmh * 0.621371),
        None => "--".to_string(),
    }
}

pub fn fmt_pressure_kpa(val: Option<f64>) -> String {
    match val {
        Some(kpa) => format!("{:.0} kPa / {:.1} PSI", kpa, kpa / 6.895),
        None => "--".to_string(),
    }
}

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

pub fn fmt_dist_cm(val: Option<u16>) -> String {
    match val {
        Some(cm) => format!("{}cm / {:.1}in", cm, cm as f64 / 2.54),
        None => "--".to_string(),
    }
}

pub fn fmt_fuel_rate(val: Option<f64>) -> String {
    match val {
        Some(lph) => format!("{:.2} L/h / {:.2} gal/h", lph, lph * 0.264172),
        None => "--".to_string(),
    }
}

pub fn clamp_percent(val: f64, max: f64) -> u16 {
    ((val / max * 100.0).clamp(0.0, 100.0)) as u16
}
