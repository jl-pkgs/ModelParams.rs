use modelparams::{ModelParams, Layers, NestedModelParams, PathParam};

// ── single-layer parameter structs ───────────────────────────────────────────

#[derive(ModelParams, Layers, Clone, Debug)]
pub struct VanGenuchten {
    #[param(bounds = (0.25, 0.50), units = "m3 m-3", default = 0.287)]
    pub theta_sat: f64,
    #[param(bounds = (0.03, 0.20), units = "m3 m-3", default = 0.075)]
    pub theta_res: f64,
    #[param(bounds = (0.002, 60.0), units = "cm h-1", default = 34.0)]
    pub ksat: f64,
    #[param(bounds = (0.002, 0.300), units = "cm-1", default = 0.027)]
    pub alpha: f64,
    #[param(bounds = (1.05, 4.00), units = "-", default = 3.96)]
    pub n: f64,
    pub m: f64,  // derived: 1 - 1/n, not optimised
}

#[derive(ModelParams, Layers, Clone, Debug)]
pub struct Campbell {
    #[param(bounds = (0.25, 0.50), units = "m3 m-3", default = 0.287)]
    pub theta_sat: f64,
    #[param(bounds = (-100.0, -5.0), units = "cm", default = -10.0)]
    pub psi_sat: f64,
    #[param(bounds = (0.002, 100.0), units = "cm h-1", default = 34.0)]
    pub ksat: f64,
    #[param(bounds = (3.0, 15.0), units = "-", default = 4.0)]
    pub b: f64,
}

#[derive(ModelParams, Layers, Clone, Debug)]
pub struct ParamThermal {
    #[param(bounds = (0.1, 10.0), units = "W m-1 K-1", default = 0.2)]
    pub kappa: f64,
    #[param(bounds = (1e6, 5e6), units = "J m-3 K-1", default = 2e6)]
    pub cv: f64,
}

// ── hydraulic union (enum) ────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum SoilHydraulic {
    VanGenuchten(VanGenuchtenLayers),
    Campbell(CampbellLayers),
}

impl NestedModelParams for SoilHydraulic {
    fn collect_params(&self, prefix: &[String]) -> Vec<PathParam> {
        match self {
            SoilHydraulic::VanGenuchten(l) => l.collect_params(prefix),
            SoilHydraulic::Campbell(l)     => l.collect_params(prefix),
        }
    }
    fn update_param(&mut self, path: &[String], value: f64) -> bool {
        match self {
            SoilHydraulic::VanGenuchten(l) => l.update_param(path, value),
            SoilHydraulic::Campbell(l)     => l.update_param(path, value),
        }
    }
}

// ── composite soil model ──────────────────────────────────────────────────────

pub struct SoilModel {
    pub n:            usize,
    pub hydraulic:    SoilHydraulic,
    pub thermal:      ParamThermalLayers,
    pub thermal_hide: ParamThermalLayers,  // hidden: not in parameters()
}

impl SoilModel {
    pub fn new_vg(n: usize) -> Self {
        Self {
            n,
            hydraulic:    SoilHydraulic::VanGenuchten(VanGenuchtenLayers::new(n)),
            thermal:      ParamThermalLayers::new(n),
            thermal_hide: ParamThermalLayers::new(n),
        }
    }

    pub fn new_campbell(n: usize) -> Self {
        Self {
            n,
            hydraulic:    SoilHydraulic::Campbell(CampbellLayers::new(n)),
            thermal:      ParamThermalLayers::new(n),
            thermal_hide: ParamThermalLayers::new(n),
        }
    }
}

impl NestedModelParams for SoilModel {
    fn collect_params(&self, prefix: &[String]) -> Vec<PathParam> {
        let mut params = Vec::new();

        let mut p = prefix.to_vec(); p.push("hydraulic".into());
        params.extend(self.hydraulic.collect_params(&p));

        let mut p = prefix.to_vec(); p.push("thermal".into());
        params.extend(self.thermal.collect_params(&p));

        // thermal_hide is intentionally skipped
        params
    }

    fn update_param(&mut self, path: &[String], value: f64) -> bool {
        if path.is_empty() { return false; }
        match path[0].as_str() {
            "hydraulic" => self.hydraulic.update_param(&path[1..], value),
            "thermal"   => self.thermal.update_param(&path[1..], value),
            _           => false,
        }
    }
}

// ── main ──────────────────────────────────────────────────────────────────────

fn main() {
    // --- VanGenuchten model, 5 layers ---
    let model = SoilModel::new_vg(5);
    println!("=== SoilModel (VanGenuchten, N=5) ===");
    model.print_nested();

    let (x0, _lb, _ub, paths) = model.opt_bounds_nested();
    println!("\n{} optimisable parameters", x0.len());
    println!("first path: {:?}", paths[0]);

    // --- Campbell model, 3 layers ---
    let model2 = SoilModel::new_campbell(3);
    println!("\n=== SoilModel (Campbell, N=3) ===");
    model2.print_nested();

    // --- update a single parameter ---
    let mut model3 = SoilModel::new_vg(2);
    let path = vec!["hydraulic".to_string(), "theta_sat".to_string(), "0".to_string()];
    model3.update_param(&path, 0.42);
    println!("\nAfter update hydraulic.theta_sat[0] = 0.42:");
    model3.print_nested();
}
