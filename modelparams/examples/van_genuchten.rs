use modelparams::{ModelParams, optim::{sceua, SceOptions}};

#[derive(ModelParams, Clone, Debug)]
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
    // m = 1 - 1/n, derived — no #[param], not optimised
    pub m: f64,
}

impl VanGenuchten {
    pub fn new() -> Self {
        let n = 3.96;
        Self { theta_sat: 0.287, theta_res: 0.075, ksat: 34.0, alpha: 0.027, n, m: 1.0 - 1.0 / n }
    }

    /// Update derived field m after optimisation.
    pub fn update_derived(&mut self) {
        self.m = 1.0 - 1.0 / self.n;
    }
}

fn main() {
    // 1. Print initial parameters
    let p = VanGenuchten::new();
    println!("=== Initial parameters ===");
    p.print_params();

    // 2. Optimise: minimise a toy cost function (just a demo)
    //    In a real case: fn cost(x) { run_model(&p_with_x); return nse(...) }
    let (x0, lb, ub) = p.opt_bounds();
    println!("\n=== Running SCE-UA (toy objective) ===");

    let target = vec![0.30, 0.10, 20.0, 0.05, 2.5]; // fake "truth"
    let cost = move |x: &[f64]| -> f64 {
        x.iter().zip(&target).map(|(xi, ti)| (xi - ti).powi(2)).sum()
    };

    let opts = SceOptions {
        max_evals: 2000,
        n_complex: 3,
        verbose: true,
        parallel: true,
        ..SceOptions::new(x0.len())
    };

    let result = sceua(cost, &x0, &lb, &ub, opts);

    // 3. Update model with optimal parameters
    let mut p_opt = p.clone();
    p_opt.update_from_opt(&result.best_x);
    p_opt.update_derived();

    println!("\n=== Optimised parameters (cost={:.6}) ===", result.best_f);
    p_opt.print_params();
    println!("n_evals = {}, exit = {:?}", result.n_evals, result.code);
}
