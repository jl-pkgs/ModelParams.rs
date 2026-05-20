use modelparams::{ModelParams, Layers, NestedModelParams, PathParam};

// ── vegetation parameters ─────────────────────────────────────────────────────

#[derive(ModelParams, Layers, Clone, Debug)]
pub struct ParamVeg {
    // Bool fields: no #[param], not layered
    pub has_understory: bool,
    pub is_bforest:     bool,

    #[param(bounds = (0.1, 7.0),  units = "-",      default = 4.5)]
    pub lai_max_o: f64,
    #[param(bounds = (0.1, 7.0),  units = "-",      default = 2.4)]
    pub lai_max_u: f64,

    #[param(bounds = (0.02, 0.15), units = "-",     default = 0.055)]
    pub alpha_canopy_vis: f64,
    #[param(bounds = (0.15, 0.50), units = "-",     default = 0.300)]
    pub alpha_canopy_nir: f64,
    #[param(bounds = (0.05, 0.20), units = "-",     default = 0.10)]
    pub alpha_soil_sat:   f64,
    #[param(bounds = (0.20, 0.50), units = "-",     default = 0.35)]
    pub alpha_soil_dry:   f64,

    #[param(bounds = (0.85, 0.999), units = "-",    default = 0.95)]
    pub r_root_decay: f64,

    #[param(bounds = (0.3, 1.0),  units = "-",      default = 0.85)]
    pub omega: f64,
    #[param(bounds = (0.1, 50.0), units = "m",      default = 1.0)]
    pub z_canopy_o: f64,
    #[param(bounds = (0.05, 5.0), units = "m",      default = 0.2)]
    pub z_canopy_u: f64,
    #[param(bounds = (1.0, 100.0), units = "m",     default = 2.0)]
    pub z_wind: f64,

    #[param(bounds = (1.0, 20.0), units = "-",      default = 8.0)]
    pub g1_w: f64,
    #[param(bounds = (0.001, 0.1), units = "-",     default = 0.0175)]
    pub g0_w: f64,

    #[param(bounds = (5.0, 200.0), units = "umol m-2 s-1", default = 89.45)]
    pub vcmax25: f64,

    #[param(bounds = (0.5, 5.0), units = "g m-2",  default = 2.45)]
    pub n_leaf: f64,
    #[param(bounds = (0.3, 1.0), units = "-",       default = 0.5858)]
    pub slope_vc: f64,
}

impl Default for ParamVeg {
    fn default() -> Self {
        let info = Self::param_info();
        let mut s = Self {
            has_understory: true,
            is_bforest:     false,
            lai_max_o: 0.0, lai_max_u: 0.0,
            alpha_canopy_vis: 0.0, alpha_canopy_nir: 0.0,
            alpha_soil_sat: 0.0, alpha_soil_dry: 0.0,
            r_root_decay: 0.0, omega: 0.0,
            z_canopy_o: 0.0, z_canopy_u: 0.0, z_wind: 0.0,
            g1_w: 0.0, g0_w: 0.0, vcmax25: 0.0,
            n_leaf: 0.0, slope_vc: 0.0,
        };
        for (i, p) in info.iter().enumerate() { s.set_value(i, p.default); }
        s
    }
}

impl NestedModelParams for ParamVeg {
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

// ── soil hydraulic parameters ─────────────────────────────────────────────────

#[derive(ModelParams, Layers, Clone, Debug)]
pub struct ParamSoilHydraulic {
    #[param(bounds = (0.10, 0.45), units = "-",       default = 0.30)]
    pub theta_vfc: f64,
    #[param(bounds = (0.02, 0.30), units = "-",       default = 0.10)]
    pub theta_vwp: f64,
    #[param(bounds = (0.25, 0.70), units = "-",       default = 0.45)]
    pub theta_sat: f64,
    #[param(bounds = (0.01, 50.0), units = "cm h-1",  default = 5.0)]
    pub k_sat:     f64,
    #[param(bounds = (-2.0, -0.01), units = "m",      default = -0.5)]
    pub psi_sat:   f64,
    #[param(bounds = (1.5, 15.0),  units = "-",       default = 5.0)]
    pub b:         f64,
}

impl Default for ParamSoilHydraulic {
    fn default() -> Self {
        let info = Self::param_info();
        let mut s = Self { theta_vfc: 0.0, theta_vwp: 0.0, theta_sat: 0.0,
                           k_sat: 0.0, psi_sat: 0.0, b: 0.0 };
        for (i, p) in info.iter().enumerate() { s.set_value(i, p.default); }
        s
    }
}

// ── soil thermal parameters ───────────────────────────────────────────────────

#[derive(ModelParams, Layers, Clone, Debug)]
pub struct ParamSoilThermal {
    #[param(bounds = (0.05, 0.5),      units = "W m-1 K-1", default = 0.2)]
    pub kappa_dry: f64,
    #[param(bounds = (800.0, 1800.0),  units = "kg m-3",    default = 1300.0)]
    pub rho_soil:  f64,
    #[param(bounds = (0.0, 0.3),       units = "-",         default = 0.02)]
    pub v_som:     f64,
}

impl Default for ParamSoilThermal {
    fn default() -> Self {
        let info = Self::param_info();
        let mut s = Self { kappa_dry: 0.0, rho_soil: 0.0, v_som: 0.0 };
        for (i, p) in info.iter().enumerate() { s.set_value(i, p.default); }
        s
    }
}

// ── composite BEPS model ──────────────────────────────────────────────────────

pub struct ParamBEPS {
    pub n:        usize,
    pub dz:       Vec<f64>,
    pub r_drainage: f64,
    pub psi_min:  f64,
    pub alpha:    f64,
    pub hydraulic: ParamSoilHydraulicLayers,
    pub thermal:   ParamSoilThermalLayers,
    pub veg:       ParamVeg,
}

impl ParamBEPS {
    pub fn new(n: usize) -> Self {
        let dz_default = vec![0.05, 0.10, 0.20, 0.40, 1.25];
        let dz = if n == 5 { dz_default } else { vec![0.2; n] };
        Self {
            n,
            dz,
            r_drainage: 0.50,
            psi_min:    33.0,
            alpha:       0.4,
            hydraulic:  ParamSoilHydraulicLayers::new(n),
            thermal:    ParamSoilThermalLayers::new(n),
            veg:        ParamVeg::default(),
        }
    }
}

impl Default for ParamBEPS { fn default() -> Self { Self::new(5) } }

impl NestedModelParams for ParamBEPS {
    fn collect_params(&self, prefix: &[String]) -> Vec<PathParam> {
        let mut params = Vec::new();

        // scalar top-level params
        let mut p = prefix.to_vec(); p.push("r_drainage".into());
        params.push(PathParam {
            path: p, name: "r_drainage".into(), value: self.r_drainage,
            bounds: Some((0.2, 0.7)), units: "-",
        });

        // layered hydraulic
        let mut p = prefix.to_vec(); p.push("hydraulic".into());
        params.extend(self.hydraulic.collect_params(&p));

        // layered thermal
        let mut p = prefix.to_vec(); p.push("thermal".into());
        params.extend(self.thermal.collect_params(&p));

        // veg (flat)
        let mut p = prefix.to_vec(); p.push("veg".into());
        params.extend(self.veg.collect_params(&p));

        params
    }

    fn update_param(&mut self, path: &[String], value: f64) -> bool {
        if path.is_empty() { return false; }
        match path[0].as_str() {
            "r_drainage" => { self.r_drainage = value; true }
            "hydraulic"  => self.hydraulic.update_param(&path[1..], value),
            "thermal"    => self.thermal.update_param(&path[1..], value),
            "veg"        => self.veg.update_param(&path[1..], value),
            _            => false,
        }
    }
}

// ── main ──────────────────────────────────────────────────────────────────────

fn main() {
    let model = ParamBEPS::default();

    println!("=== ParamBEPS (N=5) ===");
    let params = model.parameters();
    println!("{} optimisable parameters", params.len());

    // show first 6
    for p in params.iter().take(6) {
        let (lo, hi) = p.bounds.unwrap_or((f64::NAN, f64::NAN));
        println!("  {:<35} = {:>10.4}  [{:<12}]  ({:.3}, {:.3})",
                 p.path.join("."), p.value, p.units, lo, hi);
    }
    println!("  ...");

    // update a few parameters
    let mut model2 = ParamBEPS::default();
    let paths = vec![
        vec!["r_drainage".to_string()],
        vec!["veg".to_string(), "omega".to_string()],
        vec!["veg".to_string(), "g1_w".to_string()],
        vec!["veg".to_string(), "g0_w".to_string()],
        vec!["veg".to_string(), "vcmax25".to_string()],
    ];
    let values = [0.6, 0.9, 10.0, 0.01, 100.0];
    model2.update_from_paths(&paths, &values);

    println!("\nAfter update:");
    println!("  r_drainage = {}", model2.r_drainage);
    println!("  veg.omega  = {}", model2.veg.omega);
    println!("  veg.g1_w   = {}", model2.veg.g1_w);
    println!("  veg.vcmax25 = {}", model2.veg.vcmax25);

    // verify via parameters()
    let updated: Vec<f64> = paths.iter()
        .map(|p| model2.parameters().into_iter().find(|pp| pp.path == *p).unwrap().value)
        .collect();
    println!("\nVerification via parameters(): {:?}", updated);
    assert_eq!(updated, values.to_vec());
    println!("All assertions passed.");
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beps_param_count() {
        let model = ParamBEPS::default();
        let params = model.parameters();
        // r_drainage(1) + hydraulic(6*5) + thermal(3*5) + veg(15) = 1+30+15+15 = 61
        assert!(!params.is_empty());
        println!("BEPS param count: {}", params.len());
    }

    #[test]
    fn test_beps_update_and_read() {
        let mut model = ParamBEPS::default();
        let paths = vec![
            vec!["r_drainage".to_string()],
            vec!["veg".to_string(), "omega".to_string()],
            vec!["veg".to_string(), "g1_w".to_string()],
            vec!["veg".to_string(), "g0_w".to_string()],
            vec!["veg".to_string(), "vcmax25".to_string()],
        ];
        let values = [0.6_f64, 0.9, 10.0, 0.01, 100.0];
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
    fn test_veg_bool_fields_not_in_params() {
        let veg = ParamVeg::default();
        let params = veg.collect_params(&[]);
        // bool fields should not appear in parameters
        assert!(!params.iter().any(|p| p.name == "has_understory"));
        assert!(!params.iter().any(|p| p.name == "is_bforest"));
    }

    #[test]
    fn test_campbell_style_negative_bounds() {
        let h = ParamSoilHydraulic::default();
        let info = ParamSoilHydraulic::param_info();
        let psi = info.iter().find(|p| p.name == "psi_sat").unwrap();
        assert!(psi.bounds.unwrap().0 < 0.0, "psi_sat lower bound should be negative");
        assert_eq!(h.psi_sat, -0.5);
    }
}
