use modelparams::optim::{sceua, SceOptions};

fn opts(n: usize) -> SceOptions {
    SceOptions {
        kstop: 10,
        // These benchmark tests assert tight optima; avoid early stall exits.
        f_reltol: -1.0,
        x_reltol: 1e-8,
        verbose: false,
        parallel: false,
        ..SceOptions::new(n)
    }
}

// ── benchmark functions ───────────────────────────────────────────────────────

/// Goldstein-Price  Global optimum: 3.0 at (0, -1)
fn goldstein_price(x: &[f64]) -> f64 {
    let (x1, x2) = (x[0], x[1]);
    let u1 = (x1 + x2 + 1.0).powi(2);
    let u2 = 19.0 - 14.0*x1 + 3.0*x1*x1 - 14.0*x2 + 6.0*x1*x2 + 3.0*x2*x2;
    let u3 = (2.0*x1 - 3.0*x2).powi(2);
    let u4 = 18.0 - 32.0*x1 + 12.0*x1*x1 + 48.0*x2 - 36.0*x1*x2 + 27.0*x2*x2;
    (1.0 + u1*u2) * (30.0 + u3*u4)
}

/// Rosenbrock  Global optimum: 0 at (1, 1)
fn rosenbrock(x: &[f64]) -> f64 {
    let (x1, x2) = (x[0], x[1]);
    100.0*(x2 - x1*x1).powi(2) + (1.0 - x1).powi(2)
}

/// Six-hump Camelback  Global optimum: -1.031628453 at (±0.08983, ∓0.7126)
fn six_hump_camelback(x: &[f64]) -> f64 {
    let (x1, x2) = (x[0], x[1]);
    (4.0 - 2.1*x1*x1 + x1.powi(4)/3.0)*x1*x1 + x1*x2 + (-4.0 + 4.0*x2*x2)*x2*x2
}

/// Rastrigin  Global optimum: -2 at (0, 0)
fn rastrigin(x: &[f64]) -> f64 {
    x[0]*x[0] + x[1]*x[1] - (18.0*x[0]).cos() - (18.0*x[1]).cos()
}

/// Griewank (10-D)  Global optimum: 0 at origin
fn griewank(x: &[f64]) -> f64 {
    let d = 4000.0;
    let u1: f64 = x.iter().map(|xi| xi*xi / d).sum();
    let u2: f64 = x.iter().enumerate()
        .map(|(j, xi)| (xi / (j as f64 + 1.0).sqrt()).cos())
        .product();
    u1 - u2 + 1.0
}

/// Shekel (4-D)  Global optimum: -10.5364 at (4,4,4,4)
fn shekel(x: &[f64]) -> f64 {
    #[rustfmt::skip]
    let a = [
        [4.0, 4.0, 4.0, 4.0],
        [1.0, 1.0, 1.0, 1.0],
        [8.0, 8.0, 8.0, 8.0],
        [6.0, 6.0, 6.0, 6.0],
        [3.0, 7.0, 3.0, 7.0],
        [2.0, 9.0, 2.0, 9.0],
        [5.0, 5.0, 3.0, 3.0],
        [8.0, 1.0, 8.0, 1.0],
        [6.0, 2.0, 6.0, 2.0],
        [7.0, 3.6, 7.0, 3.6f64],
    ];
    let c = [0.1, 0.2, 0.2, 0.4, 0.4, 0.6, 0.3, 0.7, 0.5, 0.5f64];
    let mut f = 0.0;
    for i in 0..10 {
        let u: f64 = x.iter().enumerate().map(|(j, xj)| (xj - a[i][j]).powi(2)).sum();
        f -= 1.0 / (u + c[i]);
    }
    f
}

/// Zakharov (n-D)  Global optimum: 1 at x_i = 0.5
fn zakharov(x: &[f64]) -> f64 {
    let y1: f64 = x.iter().map(|xi| (xi - 0.5).powi(2)).sum();
    let y2: f64 = x.iter().enumerate().map(|(i, xi)| (i as f64 + 2.0) * (xi - 0.5)).sum();
    let h = 0.5 * y2;
    y1 + h*h + h.powi(4) + 1.0
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[test]
fn test_goldstein_price() {
    let bl = [-2.0, -2.0];
    let bu = [ 2.0,  2.0];
    let x0 = [ 1.0,  1.0];
    let r = sceua(goldstein_price, &x0, &bl, &bu,
                  SceOptions { max_evals: 2000, ..opts(2) });
    assert!((r.best_f - 3.0).abs() < 1e-4,
            "Goldstein-Price: expected ~3, got {}", r.best_f);
}

#[test]
fn test_rosenbrock() {
    let bl = [-5.0, -5.0];
    let bu = [ 5.0,  5.0];
    let x0 = [-1.0,  1.0];
    let r = sceua(rosenbrock, &x0, &bl, &bu,
                  SceOptions { max_evals: 10000, ..opts(2) });
    assert!(r.best_f.abs() < 1e-6,
            "Rosenbrock: expected ~0, got {}, x={:?}, code={:?}, n_evals={}",
            r.best_f, r.best_x, r.code, r.n_evals);
}

#[test]
fn test_six_hump_camelback() {
    let bl = [-5.0, -2.0];
    let bu = [ 5.0,  8.0];
    let x0 = [-0.08983, 0.7126];
    let r = sceua(six_hump_camelback, &x0, &bl, &bu, opts(2));
    assert!((r.best_f - (-1.031628453489877)).abs() < 1e-6,
            "Six-hump: expected ~-1.0316, got {}", r.best_f);
}

#[test]
fn test_rastrigin() {
    let bl = [-1.0, -1.0];
    let bu = [ 1.0,  1.0];
    let x0 = [-1.0, -1.0];
    let r = sceua(rastrigin, &x0, &bl, &bu,
                  SceOptions { max_evals: 10000, ..opts(2) });
    assert!((r.best_f - (-2.0)).abs() < 1e-3,
            "Rastrigin: expected ~-2, got {}, x={:?}, code={:?}, n_evals={}",
            r.best_f, r.best_x, r.code, r.n_evals);
}

#[test]
fn test_griewank() {
    let n = 10;
    let bl = vec![-600.0; n];
    let bu = vec![ 600.0; n];
    let x0 = vec![-1.0; n];
    let r = sceua(griewank, &x0, &bl, &bu,
                  SceOptions { max_evals: 20000, ..opts(n) });
    assert!(r.best_f.abs() < 1e-6,
            "Griewank: expected ~0, got {}", r.best_f);
}

#[test]
fn test_shekel() {
    let bl = [0.0; 4];
    let bu = [10.0; 4];
    let x0 = [4.0, 4.0, 4.0, 3.0];
    let r = sceua(shekel, &x0, &bl, &bu,
                  SceOptions { max_evals: 10000, ..opts(4) });
    assert!((r.best_f - (-10.5364098252)).abs() < 2e-5,
            "Shekel: expected ~-10.5364, got {}", r.best_f);
}

#[test]
fn test_zakharov_10d() {
    let n = 10;
    let bl = vec![-1.0; n];
    let bu = vec![ 1.0; n];
    let x0 = vec![ 0.0; n];
    let r = sceua(zakharov, &x0, &bl, &bu,
                  SceOptions { max_evals: 10000, ..opts(n) });
    // Zakharov optimum = 1.0
    assert!((r.best_f - 1.0).abs() < 1e-4,
            "Zakharov-10D: expected ~1, got {}", r.best_f);
}

#[test]
fn test_parallel_reproducibility() {
    let bl = [-5.0, -5.0];
    let bu = [ 5.0,  5.0];
    let x0 = [-1.0,  1.0];
    let base_opts = SceOptions {
        max_evals: 2000, seed: 1, verbose: false,
        kstop: 10, f_reltol: 1e-3, x_reltol: 1e-3,
        ..SceOptions::new(2)
    };

    let r1 = sceua(rosenbrock, &x0, &bl, &bu,
                   SceOptions { parallel: true,  ..base_opts.clone() });
    let r2 = sceua(rosenbrock, &x0, &bl, &bu,
                   SceOptions { parallel: true,  ..base_opts.clone() });
    let rs = sceua(rosenbrock, &x0, &bl, &bu,
                   SceOptions { parallel: false, ..base_opts.clone() });

    assert_eq!(r1.code, r2.code);
    assert_eq!(r1.best_f, r2.best_f, "parallel runs must be reproducible");
    assert_eq!(r1.best_f, rs.best_f, "parallel == serial result");
}
