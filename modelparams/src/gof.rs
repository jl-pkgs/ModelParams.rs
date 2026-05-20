/// Filter out pairs where either obs or sim is NaN/Inf.
/// Returns (obs_valid, sim_valid); returns ([], []) if all invalid.
fn valid_pairs<'a>(obs: &'a [f64], sim: &'a [f64]) -> (Vec<f64>, Vec<f64>) {
    obs.iter().zip(sim.iter())
        .filter(|(o, s)| o.is_finite() && s.is_finite())
        .map(|(&o, &s)| (o, s))
        .unzip()
}

fn mean(v: &[f64]) -> f64 { v.iter().sum::<f64>() / v.len() as f64 }
fn std(v: &[f64], m: f64) -> f64 { (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64).sqrt() }

/// Nash-Sutcliffe Efficiency  (1 = perfect, <0 worse than mean)
/// NaN pairs are skipped. Returns f64::NAN if <2 valid pairs or ss_tot = 0.
pub fn nse(obs: &[f64], sim: &[f64]) -> f64 {
    let (o, s) = valid_pairs(obs, sim);
    if o.len() < 2 { return f64::NAN; }
    let mean_o = mean(&o);
    let ss_res: f64 = o.iter().zip(&s).map(|(oi, si)| (oi - si).powi(2)).sum();
    let ss_tot: f64 = o.iter().map(|oi| (oi - mean_o).powi(2)).sum();
    if ss_tot == 0.0 { return f64::NAN; }
    1.0 - ss_res / ss_tot
}

/// Kling-Gupta Efficiency  (1 = perfect)
/// NaN pairs are skipped. Returns f64::NAN if mean_obs = 0 or std = 0.
pub fn kge(obs: &[f64], sim: &[f64]) -> f64 {
    let (o, s) = valid_pairs(obs, sim);
    if o.len() < 2 { return f64::NAN; }
    let mean_o = mean(&o);
    let mean_s = mean(&s);
    if mean_o == 0.0 { return f64::NAN; }
    let std_o = std(&o, mean_o);
    let std_s = std(&s, mean_s);
    if std_o == 0.0 || mean_s == 0.0 { return f64::NAN; }
    let n = o.len() as f64;
    let r = o.iter().zip(&s)
        .map(|(oi, si)| (oi - mean_o) * (si - mean_s))
        .sum::<f64>() / (n * std_o * std_s);
    let beta  = mean_s / mean_o;
    let gamma = (std_s / mean_s) / (std_o / mean_o);
    1.0 - ((r - 1.0).powi(2) + (beta - 1.0).powi(2) + (gamma - 1.0).powi(2)).sqrt()
}

/// Root Mean Square Error
pub fn rmse(obs: &[f64], sim: &[f64]) -> f64 {
    let (o, s) = valid_pairs(obs, sim);
    if o.is_empty() { return f64::NAN; }
    (o.iter().zip(&s).map(|(oi, si)| (oi - si).powi(2)).sum::<f64>() / o.len() as f64).sqrt()
}

/// Mean Absolute Error
pub fn mae(obs: &[f64], sim: &[f64]) -> f64 {
    let (o, s) = valid_pairs(obs, sim);
    if o.is_empty() { return f64::NAN; }
    o.iter().zip(&s).map(|(oi, si)| (oi - si).abs()).sum::<f64>() / o.len() as f64
}

/// Percent bias  (%, positive = overestimate)
pub fn pbias(obs: &[f64], sim: &[f64]) -> f64 {
    let (o, s) = valid_pairs(obs, sim);
    if o.is_empty() { return f64::NAN; }
    let sum_o: f64 = o.iter().sum();
    if sum_o == 0.0 { return f64::NAN; }
    100.0 * o.iter().zip(&s).map(|(oi, si)| oi - si).sum::<f64>() / sum_o
}

#[cfg(test)]
mod tests {
    use super::*;

    const OBS: &[f64] = &[1.0, 2.0, 3.0, 4.0, 5.0];
    const SIM: &[f64] = &[1.1, 1.9, 3.2, 3.8, 5.1];

    #[test]
    fn test_nse_perfect() {
        assert_eq!(nse(OBS, OBS), 1.0);
    }

    #[test]
    fn test_nse_normal() {
        let v = nse(OBS, SIM);
        assert!(v > 0.9 && v < 1.0, "nse={v}");
    }

    #[test]
    fn test_nse_with_nan() {
        let obs = [1.0, f64::NAN, 3.0, 4.0, 5.0];
        let sim = [1.1, 1.9,      3.2, f64::NAN, 5.1];
        // pairs (1,1.1) and (3,3.2) and (5,5.1) are valid
        let v = nse(&obs, &sim);
        assert!(v.is_finite(), "nse should be finite after NaN filtering, got {v}");
    }

    #[test]
    fn test_nse_all_nan() {
        let obs = [f64::NAN, f64::NAN];
        let sim = [1.0,      2.0];
        assert!(nse(&obs, &sim).is_nan());
    }

    #[test]
    fn test_kge_perfect() {
        let v = kge(OBS, OBS);
        assert!((v - 1.0).abs() < 1e-10, "kge={v}");
    }

    #[test]
    fn test_kge_with_nan() {
        let obs = [f64::NAN, 2.0, 3.0, 4.0, 5.0];
        let sim = [1.1,      1.9, 3.2, 3.8, 5.1];
        assert!(kge(&obs, &sim).is_finite());
    }

    #[test]
    fn test_rmse_perfect() {
        assert_eq!(rmse(OBS, OBS), 0.0);
    }

    #[test]
    fn test_rmse_with_nan() {
        let obs = [1.0, f64::NAN, 3.0];
        let sim = [1.0, 999.0,    3.0];
        assert_eq!(rmse(&obs, &sim), 0.0);  // NaN pair skipped, remaining are perfect
    }

    #[test]
    fn test_pbias_zero() {
        assert_eq!(pbias(OBS, OBS), 0.0);
    }

    #[test]
    fn test_pbias_obs_zero() {
        let obs = [0.0, 0.0];
        let sim = [1.0, 1.0];
        assert!(pbias(&obs, &sim).is_nan());
    }
}
