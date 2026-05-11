#[allow(dead_code)]
pub fn offset_trim_to_u64(data: &[f64]) -> Vec<u64> {
    data.iter()
        .map(|&v| ((v + 25.0) * 100.0).clamp(0.0, u64::MAX as f64) as u64)
        .collect()
}
