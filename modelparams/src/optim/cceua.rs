use rand::Rng;

/// Competitive Complex Evolution step.
/// s must be sorted best-first. Returns (snew, fnew, n_evals) where snew
/// replaces the worst point s[last]; caller inserts it into the complex.
pub fn cceua<F>(
    f: &F,
    s: &[Vec<f64>],
    sf: &[f64],
    bl: &[f64],
    bu: &[f64],
    rng: &mut impl Rng,
) -> (Vec<f64>, f64, usize)
where
    F: Fn(&[f64]) -> f64 + Sync,
{
    let n = s.len();
    let n_param = bl.len();
    let sw = &s[n - 1];
    let fw = sf[n - 1];

    // centroid of all points except the worst
    let mut ce = vec![0.0_f64; n_param];
    for i in 0..n - 1 {
        for j in 0..n_param { ce[j] += s[i][j]; }
    }
    for j in 0..n_param { ce[j] /= (n - 1) as f64; }

    // reflection: snew = 2*ce - sw; use random if outside bounds
    let mut snew: Vec<f64> = (0..n_param).map(|j| 2.0 * ce[j] - sw[j]).collect();
    if snew.iter().zip(bl).zip(bu).any(|((&v, &lo), &hi)| v < lo || v > hi) {
        snew = (0..n_param).map(|j| rng.gen_range(bl[j]..=bu[j])).collect();
    }
    let fnew = f(&snew);
    if fnew <= fw { return (snew, fnew, 1); }

    // contraction
    snew = (0..n_param).map(|j| sw[j] + 0.5 * (ce[j] - sw[j])).collect();
    let fnew = f(&snew);
    if fnew <= fw { return (snew, fnew, 2); }

    // random replacement
    snew = (0..n_param).map(|j| rng.gen_range(bl[j]..=bu[j])).collect();
    let fnew = f(&snew);
    (snew, fnew, 3)
}

pub fn sort_by_cost(pts: &mut Vec<Vec<f64>>, costs: &mut Vec<f64>) {
    let mut idx: Vec<usize> = (0..costs.len()).collect();
    idx.sort_by(|&a, &b| costs[a].partial_cmp(&costs[b]).unwrap());
    let ps: Vec<Vec<f64>> = idx.iter().map(|&i| pts[i].clone()).collect();
    let fs: Vec<f64>      = idx.iter().map(|&i| costs[i]).collect();
    *pts   = ps;
    *costs = fs;
}
