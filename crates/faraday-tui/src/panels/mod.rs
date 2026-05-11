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

pub fn clamp_percent(val: f64, max: f64) -> u16 {
    ((val / max * 100.0).clamp(0.0, 100.0)) as u16
}
