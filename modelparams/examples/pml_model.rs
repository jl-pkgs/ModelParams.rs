use modelparams::{ModelParams, NestedModelParams, PathParam};

// ── water constraint model ────────────────────────────────────────────────────

/// Zhang 2019 VPD-based water constraint.
/// Global optimum corresponds to no water stress (β = 1).
#[derive(ModelParams, Clone, Debug)]
pub struct BetaGppZhang2019 {
    #[param(bounds = (0.65, 1.5), units = "kPa", default = 0.9)]
    pub vpd_min: f64,
    #[param(bounds = (3.50, 6.5), units = "kPa", default = 4.0)]
    pub vpd_max: f64,
}

impl Default for BetaGppZhang2019 {
    fn default() -> Self {
        let info = Self::param_info();
        let mut s = Self { vpd_min: 0.0, vpd_max: 0.0 };
        for (i, p) in info.iter().enumerate() { s.set_value(i, p.default); }
        s
    }
}

impl NestedModelParams for BetaGppZhang2019 {
    fn collect_params(&self, prefix: &[String]) -> Vec<PathParam> {
        Self::param_info().iter().zip(self.get_values().iter())
            .filter(|(p, _)| p.bounds.is_some())
            .map(|(p, &v)| {
                let mut path = prefix.to_vec();
                path.push(p.name.to_string());
                PathParam { path, name: p.name.to_string(), value: v,
                            bounds: p.bounds, units: p.units }
            }).collect()
    }
    fn update_param(&mut self, path: &[String], value: f64) -> bool {
        if path.is_empty() { return false; }
        let info = Self::param_info();
        for (i, p) in info.iter().enumerate() {
            if p.name == path[0] { self.set_value(i, value); return true; }
        }
        false
    }
}

// ── photosynthesis model ──────────────────────────────────────────────────────

/// Rong 2018 light-use-efficiency photosynthesis model.
/// Owns scalar params plus a nested water-constraint sub-model.
#[derive(ModelParams, Clone, Debug)]
pub struct PhotosynthesisRong2018 {
    #[param(bounds = (0.01, 0.10), units = "umol CO2 [umol PAR]-1", default = 0.06)]
    pub alpha: f64,

    #[param(bounds = (0.01, 0.07), units = "umol m-2 s-1 [umol m-2 s-1]-1", default = 0.04)]
    pub eta: f64,

    #[param(bounds = (5.00, 120.00), units = "umol m-2 s-1", default = 50.0)]
    pub vcmax25: f64,

    #[param(bounds = (0.0, 5.0), units = "-", default = 2.0)]
    pub d_pc: f64,

    #[param(bounds = (0.10, 1.0), units = "-", default = 0.45)]
    pub kq: f64,

    // watercons is a nested model, not a scalar param - no #[param]
    pub watercons: BetaGppZhang2019,
}

impl Default for PhotosynthesisRong2018 {
    fn default() -> Self {
        let info = Self::param_info();
        let mut s = Self {
            alpha: 0.0, eta: 0.0, vcmax25: 0.0, d_pc: 0.0, kq: 0.0,
            watercons: BetaGppZhang2019::default(),
        };
        for (i, p) in info.iter().enumerate() { s.set_value(i, p.default); }
        s
    }
}

impl NestedModelParams for PhotosynthesisRong2018 {
    fn collect_params(&self, prefix: &[String]) -> Vec<PathParam> {
        let mut params = Vec::new();

        // own scalar params
        for (p, &v) in Self::param_info().iter().zip(self.get_values().iter()) {
            if p.bounds.is_some() {
                let mut path = prefix.to_vec();
                path.push(p.name.to_string());
                params.push(PathParam {
                    path, name: p.name.to_string(), value: v,
                    bounds: p.bounds, units: p.units,
                });
            }
        }

        // nested watercons
        let mut p = prefix.to_vec();
        p.push("watercons".into());
        params.extend(self.watercons.collect_params(&p));

        params
    }

    fn update_param(&mut self, path: &[String], value: f64) -> bool {
        if path.is_empty() { return false; }
        if path[0] == "watercons" {
            return self.watercons.update_param(&path[1..], value);
        }
        let info = Self::param_info();
        for (i, p) in info.iter().enumerate() {
            if p.name == path[0] { self.set_value(i, value); return true; }
        }
        false
    }
}

// ── main ──────────────────────────────────────────────────────────────────────

fn main() {
    let model = PhotosynthesisRong2018::default();

    println!("=== PhotosynthesisRong2018 ===");
    model.print_nested();

    let params = model.parameters();
    println!("\n{} optimisable parameters", params.len());

    let units: std::collections::HashSet<&str> = params.iter().map(|p| p.units).collect();
    println!("Distinct units ({}):", units.len());
    for u in &units { println!("  {}", u); }

    // update kq, vcmax25, watercons.vpd_min
    let mut model2 = PhotosynthesisRong2018::default();
    let paths = vec![
        vec!["kq".to_string()],
        vec!["vcmax25".to_string()],
        vec!["watercons".to_string(), "vpd_min".to_string()],
    ];
    let values = [0.6_f64, 10.0, 0.8];
    model2.update_from_paths(&paths, &values);

    println!("\nAfter update:");
    println!("  kq           = {}", model2.kq);
    println!("  vcmax25      = {}", model2.vcmax25);
    println!("  watercons.vpd_min = {}", model2.watercons.vpd_min);
    assert!((model2.kq - 0.6).abs() < 1e-10);
    assert!((model2.vcmax25 - 10.0).abs() < 1e-10);
    assert!((model2.watercons.vpd_min - 0.8).abs() < 1e-10);
    println!("All assertions passed.");
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_param_count() {
        let model = PhotosynthesisRong2018::default();
        let params = model.parameters();
        // alpha, eta, vcmax25, d_pc, kq (5) + vpd_min, vpd_max (2) = 7
        assert_eq!(params.len(), 7);
    }

    #[test]
    fn test_distinct_units() {
        let model = PhotosynthesisRong2018::default();
        let params = model.parameters();
        let units: HashSet<&str> = params.iter().map(|p| p.units).collect();
        // "umol CO2 [umol PAR]-1", "umol m-2 s-1 [umol m-2 s-1]-1",
        // "umol m-2 s-1", "-", "kPa"  → 5 distinct
        assert_eq!(units.len(), 5, "expected 5 distinct units, got {:?}", units);
    }

    #[test]
    fn test_update_nested_watercons() {
        let mut model = PhotosynthesisRong2018::default();
        let paths = vec![
            vec!["kq".to_string()],
            vec!["vcmax25".to_string()],
            vec!["watercons".to_string(), "vpd_min".to_string()],
        ];
        let values = [0.6_f64, 10.0, 0.8];
        model.update_from_paths(&paths, &values);

        assert!((model.kq - 0.6).abs() < 1e-10);
        assert!((model.vcmax25 - 10.0).abs() < 1e-10);
        assert!((model.watercons.vpd_min - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_read_back_via_parameters() {
        let mut model = PhotosynthesisRong2018::default();
        let paths = vec![
            vec!["kq".to_string()],
            vec!["vcmax25".to_string()],
            vec!["watercons".to_string(), "vpd_min".to_string()],
        ];
        let values = [0.6_f64, 10.0, 0.8];
        model.update_from_paths(&paths, &values);

        let all_params = model.parameters();
        for (path, &expected) in paths.iter().zip(values.iter()) {
            let p = all_params.iter().find(|pp| pp.path == *path)
                .unwrap_or_else(|| panic!("path {:?} not found", path));
            assert!((p.value - expected).abs() < 1e-10,
                    "path {:?}: expected {}, got {}", path, expected, p.value);
        }
    }

    #[test]
    fn test_watercons_not_in_own_params() {
        // watercons has no #[param] so it should not appear in ModelParams::param_info()
        let info = PhotosynthesisRong2018::param_info();
        assert!(!info.iter().any(|p| p.name == "watercons"));
    }
}
